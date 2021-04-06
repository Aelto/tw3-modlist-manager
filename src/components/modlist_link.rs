use maud::{html, Markup};

pub fn modlist_link(name: &str) -> Markup {
  html! {
    a href={"/modlist/" (name)} { (name) }
  }
}
