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
  
  let mut modlists = match visibility {
    Some(v) => ModList::get_all()
      .iter()
      .map(ModList::read_metadata_from_disk_copy)
      .filter(Result::is_ok)
      .map(Result::unwrap)
      .filter(|modlist| modlist.visibility == v)
      .collect(),
    None => ModList::get_all()
  };


  for i in 0..modlists.len() {
    if let Err(error) = modlists[i].read_metadata_from_disk() {
      let content = html! {
        h1 { "Could not read modlist metadata" }
        p { (error) }
      };
      let view = components::page("root", &content);
    
      return HttpResponse::Ok()
      .content_type("text/html")
      .body(view.into_string())
    }
  }


  let mut installable_modlists = Vec::new();
  let mut installable_modlists_visibility_levels = std::collections::HashSet::new();

  for i in 0..modlists.len() {
    if modlists[i].name == "vanilla" || modlists[i].has_modlist_imported("vanilla") {
      installable_modlists.push(i);
      installable_modlists_visibility_levels.insert(modlists[i].visibility);
    }
  }

  let mut shared_modlists = Vec::new();
  let mut shared_modlists_visibility_levels = std::collections::HashSet::new();

  for i in 0..modlists.len() {
    if modlists[i].name != "vanilla" && !modlists[i].has_modlist_imported("vanilla") {
      shared_modlists.push(i);
      shared_modlists_visibility_levels.insert(modlists[i].visibility);
    }
  }

  let mut shared_levels = shared_modlists_visibility_levels.into_iter().collect::<Vec<i64>>();
  shared_levels.sort();
  shared_levels.reverse();

  let mut installable_levels = installable_modlists_visibility_levels.into_iter().collect::<Vec<i64>>();
  installable_levels.sort();
  installable_levels.reverse();

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
        // div class="row flex-center" {
        //   a href={"?visibility=" (visibility_down)} { "<" }
        //   span title="This allows you to navigate through modlists with different visibility levels" { (visibility) };
        //   a href={"?visibility=" (visibility_up)} { ">" }
        // }

        div class="modlist-containers" {

          div class="column" {
            h2 { "Shared modlists" }

            ul class="level-list" {
              @for level in &shared_levels {
                li class="level-listing" {
                  h3 { (level) }
  
                  ul {
                    @for index in &shared_modlists {
                      @if &modlists[*index].visibility == level {
                        li class="modlist" {
                          a title="you cannot install this modlist because it doesn't import the vanilla modlist" href={"/modlist/" (&modlists[*index].name)} { (&modlists[*index].name) }
                        }
                      }
                    }
                  }
                }
              }
            }
          }

          div class="column" {
            h2 { "Installable modlists" }

            ul class="level-list" {
              @for level in &installable_levels {
                li class="level-listing" {
                  h3 { (level) }
  
                  ul {
                    @for index in &installable_modlists {
                      @if &modlists[*index].visibility == level {
                        li class="modlist" {
                          a href={"/modlist/" (&modlists[*index].name)} { (&modlists[*index].name) }

                          @if &modlists[*index].name == "vanilla" || modlists[*index].has_modlist_imported("vanilla") {
                            form method="post" action="/api/modlist/install" {
                              input type="hidden" name="name" value=(&modlists[*index].name);
                              
                              input type="submit" value="install";
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }

          div class="column" {
            form class="create" method="post" action="/api/modlist/create" {
              h2 { "New modlist" }
              input type="text" name="modlist_name" placeholder="modlist's name";
              input type="submit" value="new";
            }
          }

        }
      }
    }

    style type="text/css" { (get_stylesheet()) }
  };

  let view = components::page("modlists", &content);
  
  HttpResponse::Ok()
  .content_type("text/html")
  .body(view.into_string())
}

fn get_stylesheet() -> String {
  "
    ul {
      padding-left: 0;
    }

    .create input + input {
      margin-left: 8px;
    }

    .modlist-containers {
      display: flex;
      flex-direction: row;
      justify-content: space-evenly;
    }

    .modlist-containers .column + .column {
      margin-left: 2em;
    }

    .modlist-containers .column .level-list {
      list-style: none;
    }

    .modlist-containers .column .level-list h3 {
      
      margin: 0;
    }

    .modlist-containers .column .level-listing ul {
      padding-left: 1em;
      margin-left: 5px;
      border-left: solid 1px white;
      list-style: none;
    }

    .modlist-containers .modlist {
      position: relative;
      display: flex;
      justify-content: space-between;
    }

    .modlist-containers .modlist:before {
      content: '';
      position: absolute;
      border-bottom: solid 1px white;
      width: 13px;
      top: 50%;
      left: -23px;
    }

    .modlist:last-child::after {
      content: '';
      background: #171413;
      width: 20px;
      height: 50%;
      position: absolute;
      top: calc(50% + 1px);
      left: -24px;
    }
  ".to_owned()
}