use crate::components;
use crate::models::modlist::ModList;

use maud::html;
use actix_web::web::HttpRequest;
use actix_web::{HttpResponse};

pub async fn render(_req: HttpRequest) -> HttpResponse {
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

        div class="row flex-end" {
          form method="post" action="/api/program/exit" {
            input type="submit" class="text-style" value="exit" onclick="setTimeout(() => window.close(), 1000);";
          }
        }
      }
    }
  };

  let view = components::page("modlists", &content);
  
  HttpResponse::Ok()
  .content_type("text/html")
  .body(view.into_string())
}