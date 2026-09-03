use leptos::prelude::*;
use leptos_use::core::IntoElementMaybeSignal;
use leptos_use::use_event_listener;
use leptos::ev::{blur, focus};

pub fn use_element_focus<El, M>(
    el: El,
) -> Signal<bool>
where
    El: IntoElementMaybeSignal<web_sys::EventTarget, M>
{
    let (is_focused, set_focused) = signal(false);
    let el = el.into_element_maybe_signal();
    let _ = use_event_listener(el, blur, move |_| set_focused(false));
    let _ = use_event_listener(el, focus, move |_| set_focused(true));

    is_focused.into()
}


pub(crate) fn prepend_relative_url(relative_url: &str) -> String {
    let document_head = document().head().unwrap();
    let meta_tags = document_head.get_elements_by_tag_name("meta");
    let mut domain_path = "".to_owned();
    for i in 0..meta_tags.length() {
        let Some(meta_tag) = meta_tags.item(i) else {break};
        let Some(name_attr) = meta_tag.get_attribute("name") else {continue};
        if name_attr != "domain-path" {continue};
        domain_path = meta_tag.get_attribute("content").unwrap();
    }
    domain_path + relative_url
}