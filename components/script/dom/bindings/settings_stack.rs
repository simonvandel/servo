/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JS, Root};
use dom::globalscope::GlobalScope;
use std::cell::RefCell;
use js::jsapi::GetScriptedCallerGlobal;
use js::jsapi::JSTracer;
use js::rust::Runtime;
use dom::bindings::trace::JSTraceable;

thread_local!(static STACK: RefCell<Vec<StackEntry>> = RefCell::new(Vec::new()));

#[derive(PartialEq, Eq, Debug, JSTraceable)]
enum StackEntryKind {
    Incumbent,
    Entry,
}

#[allow(unrooted_must_root)]
#[derive(JSTraceable)]
struct StackEntry {
    global: JS<GlobalScope>,
    kind: StackEntryKind,
}

/// Traces the script settings stack.
pub unsafe fn trace(tracer: *mut JSTracer) {
    STACK.with(|stack| {
        stack.borrow().trace(tracer);
    })
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct AutoEntryScript {
    global: *const GlobalScope,
}

impl AutoEntryScript {
    /// https://html.spec.whatwg.org/multipage/#prepare-to-run-script
    pub fn new(global: &GlobalScope) -> Self {
        STACK.with(|stack| {
            trace!("Prepare to run script with {:p}", global);
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: JS::from_ref(global),
                kind: StackEntryKind::Entry,
            });
            AutoEntryScript {
                global: global as *const _,
            }
        })
    }
}

impl Drop for AutoEntryScript {
    /// https://html.spec.whatwg.org/multipage/#clean-up-after-running-script
    fn drop(&mut self) {
        STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let entry = stack.pop().unwrap();
            assert_eq!(&*entry.global as *const GlobalScope,
                       self.global,
                       "Dropped AutoEntryScript out of order.");
            assert_eq!(entry.kind, StackEntryKind::Entry);
            trace!("Clean up after running script with {:p}", self.global);
        })
    }
}

/// Returns the ["entry"] global object.
///
/// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
pub fn entry_global() -> Root<GlobalScope> {
    STACK.with(|stack| {
        stack.borrow()
             .iter()
             .rev()
             .find(|entry| entry.kind == StackEntryKind::Entry)
             .map(|entry| Root::from_ref(&*entry.global))
    }).unwrap()
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct AutoIncumbentScript {
    global: *const GlobalScope,
}

impl AutoIncumbentScript {
    /// https://html.spec.whatwg.org/multipage/#prepare-to-run-a-callback
    pub fn new(global: &GlobalScope) -> Self {
        STACK.with(|stack| {
            trace!("Prepare to run a callback with {:p}", global);
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: JS::from_ref(global),
                kind: StackEntryKind::Incumbent,
            });
            AutoIncumbentScript {
                global: global as *const _,
            }
        })
    }
}

impl Drop for AutoIncumbentScript {
    /// https://html.spec.whatwg.org/multipage/#clean-up-after-running-a-callback
    fn drop(&mut self) {
        STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let entry = stack.pop().unwrap();
            assert_eq!(&*entry.global as *const GlobalScope,
                       self.global,
                       "Dropped AutoIncumbentScript out of order.");
            assert_eq!(entry.kind, StackEntryKind::Incumbent);
            trace!("Clean up after running a callback with {:p}", self.global);
        })
    }
}

/// Returns the ["incumbent"] global object.
///
/// ["incumbent"]: https://html.spec.whatwg.org/multipage/#incumbent
pub fn incumbent_global() -> Root<GlobalScope> {
    // See what the JS engine has to say. If we've got a scripted caller
    // override in place, the JS engine will lie to us and pretend that
    // there's nothing on the JS stack, which will cause us to check the
    // incumbent script stack below.
    unsafe {
        let cx = Runtime::get();
        assert!(!cx.is_null());
        let global = GetScriptedCallerGlobal(cx);
        if !global.is_null() {
            return GlobalScope::from_object(global);
        }
    }

    // Ok, nothing from the JS engine. Let's use whatever's on the explicit stack.
    STACK.with(|stack| {
        stack.borrow()
             .last()
             .map(|entry| Root::from_ref(&*entry.global))
    }).unwrap()
}