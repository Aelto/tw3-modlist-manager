use maud::{html, Markup};

pub fn mod_display(name: &str, modlist_name: &str, is_modlist_packed: bool) -> Markup {
  let mark_as_disabled = !name.starts_with("mod");

  html! {
    @if mark_as_disabled {
      span.folder-display.disabled-folder { (name) }
    }
    @else {
      span.folder-display {
        (name)
      }
    }

    @if !is_modlist_packed {
      a href={"/modlist/"(modlist_name)"/edit/mods/"(name)} { "edit" }
    }
    @else {
      a.disabled title="You cannot edit this file because the modlist it's in is currently packed." { "edit" }
    }
  }
}
