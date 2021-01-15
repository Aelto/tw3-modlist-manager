use crate::components;
use crate::models::modlist::ModList;

use maud::html;
use actix_web::web::HttpRequest;
use actix_web::{HttpResponse};

pub async fn render(req: HttpRequest) -> HttpResponse {
  let modlists = ModList::get_all();

  // if there is no vanilla modlist, force a call to initialize
  let should_initialize = !modlists
    .iter()
    .any(|modlist| modlist.name == "vanilla");

  let content = html! {
    section {

      @if should_initialize {
        form method="post" action="/api/modlist/initialize" {
          input type="submit" value="initialize";
        }
      }
      @else {
        form method="post" action="/api/modlist/create" {
          input type="text" name="modlist_name";
          input type="submit" value="new";
        }
  
        ul {
          @for modlist in &modlists {
            li {
              a href={"/modlist/" (modlist.name)} { (modlist.name) }
  
              form method="post" action="/api/modlist/install" {
                input type="hidden" name="name" value=(modlist.name);
                
                input type="submit" value="install";
              }
            }
          }
        }
      }
    }
  };

  let view = components::page("root", &content);
  
  HttpResponse::Ok()
  .content_type("text/html")
  .body(view.into_string())
}