use maud::{html, Markup};

pub fn modlist_link(name: &str) -> Markup {
  let mark_as_disabled = name.contains("~");

  html! {
    a href={"/modlist/" (name)} { (name) }
  }
}
