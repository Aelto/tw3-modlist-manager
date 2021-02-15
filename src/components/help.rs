use maud::{html, Markup};

pub fn help(message: &str) -> Markup {
  html! {
    span.help title=(message) { "?" }
  }
}
