use crate::components;
use crate::models::modlist::ModList;

use maud::html;
use actix_web::web::HttpRequest;
use actix_web::{HttpResponse};

pub async fn render(req: HttpRequest) -> HttpResponse {
  let modlist_name = req
    .match_info()
    .get("modlist_name")
    .unwrap_or("__unknown__");

  let some_modlist = ModList::get_by_name(modlist_name);

  if some_modlist.is_none() {
    let content = html! {
      h1 { "no such modlist" }
    };
    let view = components::page("root", &content);
  
    return HttpResponse::Ok()
    .content_type("text/html")
    .body(view.into_string())
  }

  let mut modlist = some_modlist.unwrap();
  
  if let Err(error) = modlist.read_imports_from_disk() {
    let content = html! {
      h1 { "Could not read modlist metadata" }
      p { (error) }
    };
    let view = components::page("root", &content);
  
    return HttpResponse::Ok()
    .content_type("text/html")
    .body(view.into_string())
  }

  let all_modlists = ModList::get_all();
  let all_modlists = all_modlists
    .iter()
    .filter(|ml| modlist.name != ml.name);

  let content = html! {
    section {
      h1 {
        (modlist.name)
        form method="post" action="/api/modlist/view" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="submit" value="view" class="small text-style";
        }
      }

      div {
        // h2 { "imported modlists" }

        div class="row even" {
          form method="post" action="/api/modlist/load-imports" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="submit" value="load imports";
          }
  
          form method="post" action="/api/modlist/unload-imports" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="submit" value="unload imports";
          }
        }

        form method="post" action="/api/modlist/import" {
          fieldset {
            legend { "import a modlist" }

            input type="hidden" name="modlist_name" value=(modlist.name);
            
            div class="row even" {
              select name="imported_name" {
                @for ml in all_modlists {
                  option value=(ml.name) { (ml.name) }
                }
              }
  
              input type="submit" value="import";
            }
          }
        }

        ul {
          @for imported_modlist in modlist.imported_modlists {
            li {
              a href={"/modlist/" (imported_modlist)} { (imported_modlist) };

              span class="row" {
                form method="post" action="/api/modlist/move-import-up" {
                  input type="hidden" name="modlist_name" value=(modlist.name);
                  input type="hidden" name="imported_modlist_name" value=(imported_modlist);
  
                  input type="submit" class="rotate-90-clockwise text-style" value="<";
                }
  
                form method="post" action="/api/modlist/move-import-down" {
                  input type="hidden" name="modlist_name" value=(modlist.name);
                  input type="hidden" name="imported_modlist_name" value=(imported_modlist);
  
                  input type="submit" class="rotate-90-clockwise text-style" value=">";
                }
  
                form method="post" action="/api/modlist/remove-import" {
                  input type="hidden" name="modlist_name" value=(modlist.name);
                  input type="hidden" name="imported_name" value=(imported_modlist);
  
                  input type="submit" value="remove";
                }
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