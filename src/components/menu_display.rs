use maud::{html, Markup};

pub fn menu_display(name: &str, modlist_name: &str) -> Markup {
  let mark_as_disabled = !name.ends_with(".xml");

  html! {
    @if mark_as_disabled {
      span.folder-display.disabled-folder { (name) }
    }
    @else {
      span.folder-display {
        (name)
      }
    }
    
    a href={"/modlist/"(modlist_name)"/edit/menus/"(name)}  { "edit" }
  }
}