use maud::{html, Markup};

pub fn dlc_display(name: &str, modlist_name: &str) -> Markup {
  let mark_as_disabled = !name.starts_with("dlc");

  html! {
    @if mark_as_disabled {
      span.folder-display.disabled-folder { (name) }
    }
    @else {
      span.folder-display { (name) }
    }
    
    a href={"/modlist/"(modlist_name)"/edit/dlcs/"(name)} { "edit" }
  }
}