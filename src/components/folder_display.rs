use maud::{html, Markup};

pub fn folder_display(name: &str) -> Markup {
  let mark_as_disabled = name.starts_with("~");

  html! {
    @if mark_as_disabled {
      span.disabled-folder { (name) }
    }
    @else {
      span.folder-display {
        (name)
      }

      a { "edit" }
    }

  }
}