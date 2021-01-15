use maud::{html, Markup};

pub fn menu(page_title: &str) -> Markup {
  html! {
    div.menu {
      (menu_link(&html! { "modlist manager" }, "/", page_title, "root"))
    }
  }
}

fn menu_link(text: &Markup, href: &str, page_title: &str, match_str: &str) -> Markup {
  html! {
    @if page_title == match_str {
      a class="active" href=(href) { (text) }
    } @else {
      a href=(href) { (text) }
    }
  }
}