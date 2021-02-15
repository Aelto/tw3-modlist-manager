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
  
  if let Err(error) = modlist.read_metadata_from_disk() {
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

  let does_import_a_modlist = modlist.imported_modlists
    .iter()
    .filter(|modlist_name| modlist_name != &"vanilla")
    .count() > 0;

  let packing_help = "
    Packing transforms a modlist in a way that allows you to pre-merge the mods
    and then re-use the merged mods directly the next time you import the modlist.

    What it does:
      It creates a new fake mod with all the script mods there are curerntly in
      the modlist and sets the mod as a high-priority mod so it is loaded before
      the regular mods. And then, it takes the current mergedfiles folder (your merged mods)
      and places its content into the fake mod too. So you now have a fake mod
      that is actually all your mods but already merged.

    So now you can import the modlist and merge the full modlist without having
    to merge the mods again.
    
    You can always revert this change by clicking the unpack button after you
    packed a modlist.
    
    IMPORTANT: You should always pack AFTER merging the mods.
  ";

  let unpacking_help = "
    the unpacking option is available when the modlist manager detects the modlist
    is in a packaged state. The modlist is in the packaged state when the pack-backup
    folder is found.

    unpacking a modlist retrieves all the backed-up mods from before the modlist
    was packaged and restores everything including the mergedfiles. Note that if
    you made changes to mods that were packaged AFTER packaging the modlist, the
    changes will be erased. New mods are safe though.

    use this option only if you want to change the modlist, add or remove a mod,
    or make changes to the merges.
    You will then have the option to pack the modlist again if you want to.
  ";

  let packing_blocked_help = "
  Be aware that packing a modlist changes lots of things to
  the mods that are installed in the modlist. So if your modlist happens to
  import another modlist and you pack it, the imported modlist will be modified
  too until you unpack it. The vanilla modlist being the exception as it doesn't
  have any mods.
  So in theory you should not pack a modlist that imports other modlists, and you
  should organize your modlists in a way where you compose lots of small packed
  modlists to build larger modlists. But the larger modlist should NEVER be packed.
  ";

  let content = html! {
    section {
      h1 {
        (modlist.name)
      }

      div class="row even" {
        label title="The visibility level you to filter modlists based on an arbitrary value you set.
        The value can be either positive or negative and is set at 0 by default" { "visibility:" }
        span {(modlist.visibility)};

        form method="post" action="/api/modlist/visibility-down" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="imported_modlist_name" value="-1";

          input type="submit" class="rotate-90-clockwise text-style" value=">";
        }

        form method="post" action="/api/modlist/visibility-up" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="imported_modlist_name" value="1";

          input type="submit" class="rotate-90-clockwise text-style" value="<";
        }
      }

      div class="row even" {
        form method="post" action="/api/modlist/view" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="submit" value="view" class="text-style";
        }

        @if !modlist.is_packed() {
          form method="post" action="/api/modlist/merge" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="submit" value="merge" class="text-style";
          }
        }

        @if modlist.is_packed() {
          form method="post" action="/api/modlist/unpack" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="submit" value="unpack" class="text-style" title=(unpacking_help);
          }
        } @else {
          @if does_import_a_modlist {
            form {
              input type="submit" value="pack" class="text-style" disabled="true" title=(packing_blocked_help);
            }
          } @else {
            form method="post" action="/api/modlist/pack" {
              input type="hidden" name="modlist_name" value=(modlist.name);
              input type="submit" value="pack" class="text-style" title=(packing_help);
            }
          }
        }
      }

      section class="imports" {
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

      style type="text/css" { (get_stylesheet()) }
    }
  };

  let view = components::page("root", &content);
  
  HttpResponse::Ok()
  .content_type("text/html")
  .body(view.into_string())
}

fn get_stylesheet() -> String {
  "
    h1 {
      margin-bottom: 0;
    }

    section.imports {
      margin-top: 3em;
    }
  ".to_owned()
}