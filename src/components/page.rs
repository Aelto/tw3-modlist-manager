use crate::components::header::header;
use crate::components::menu::menu;
use maud::Render;
use maud::{html, Markup, DOCTYPE};

pub fn page(page_title: &str, page_content: &Markup) -> Markup {
  html! {
    (DOCTYPE)
    html lang="en" {
      (header(page_title))

      body {
        (menu(page_title))

        div id="content" {
          (page_content.render())
        }
      }
    }
  }
}
