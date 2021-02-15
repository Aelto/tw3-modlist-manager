use crate::components;
use crate::models::modlist::ModList;

use maud::html;
use actix_web::web::HttpRequest;
use actix_web::{HttpResponse};

pub async fn render(req: HttpRequest) -> HttpResponse {
  
  let query = req.query_string();
  let query = qstring::QString::from(query);
  
  let visibility = query.get("visibility")
    .and_then(|n| n.parse::<i64>().ok());
  
  let modlists = match visibility {
    Some(v) => ModList::get_all()
      .iter()
      .map(ModList::read_metadata_from_disk_copy)
      .filter(Result::is_ok)
      .map(Result::unwrap)
      .filter(|modlist| modlist.visibility == v)
      .collect(),
    None => ModList::get_all()
  };

  // now that we don't need to know if it's Some or None, use a default value
  let visibility = visibility.unwrap_or(0);
  let visibility_up = visibility + 1;
  let visibility_down = visibility - 1;

  // if there is no vanilla modlist, force a call to initialize
  let should_initialize = ModList::get_by_name("vanilla")
    .is_none();

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

        br;

        div class="row flex-center" {
          a href={"?visibility=" (visibility_down)} { "<" }
          span title="This allows you to navigate through modlists with different visibility levels" { (visibility) };
          a href={"?visibility=" (visibility_up)} { ">" }
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