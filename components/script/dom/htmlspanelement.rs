/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLSpanElementBinding;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use html5ever_atoms::LocalName;

#[dom_struct]
pub struct HTMLSpanElement {
    htmlelement: HTMLElement
}

impl HTMLSpanElement {
    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document) -> HTMLSpanElement {
        HTMLSpanElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLSpanElement> {
        Node::reflect_node(box HTMLSpanElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLSpanElementBinding::Wrap)
    }
}
