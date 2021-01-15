use maud::{html, Markup, DOCTYPE};
use crate::components::header::header;
use crate::components::menu::menu;

pub fn page(page_title: &str, page_content: &Markup) -> Markup {
  html! {
    (DOCTYPE)
    html lang="en" {
      (header(page_title))

      body {
        (menu(page_title))

        div id="content" {
          (page_content)
        }
      }
    }
  }
}

pub fn page_without_menu(page_title: &str, page_content: &Markup) -> Markup {
  html! {
    (DOCTYPE)
    html lang="en" {
      (header(page_title))

      body {
        div id="content" {
          (page_content)
        }
      }
    }
  }
}