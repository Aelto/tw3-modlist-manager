use maud::{html, Markup};

pub fn menu(page_title: &str) -> Markup {
  html! {
    div.menu {
      (menu_link(&html! {
        "modlist"
        br;
        "manager"
      }, "/", page_title, "root"))

      form method="post" action="/api/program/exit" onsubmit="setTimeout(() => window.close(), 1000)" {
        input type="submit" class="text-style" value="exit" style="
          position: absolute;
          top: 10px;
          right: 10px;
        ";
      }
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