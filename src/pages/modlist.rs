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
    let view = components::page(&format!("modlist - {}", modlist_name), &content);
  
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
    is in a packaged state. The modlist is considered in the packaged state when
    a mod with the same name as the modlist is found in the modlist.

    unpacking a modlist enables all the disabled script mods to the state before
    the modlist was packaged and restores everything including the mergedfiles.
    Note that if you made changes to mods that were packaged AFTER packaging the
    modlist, the changes will be erased. New mods are safe though.

    use this option only if you want to change the modlist, add or remove a mod,
    or make changes to the merges. You will then have the option to pack the
    modlist again if you want to.
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

  let visibility_level_help = "
This number is the `visibility level` of the modlist. You will find the same number
in the home page, which you can use to filter the modlists per visibility level.
The value can either be positive or negative, and it's up to you to create a hierarchy
you understand. For example, you could set modlists you install at level 1 and modlists
that are imported by other modlists at level -1. This way, when you're on the home page
you can go left and see the imported modlists or go right and see the modlists you
can install.
  ";

  let merge_help = "
opens the scriptmerger for this modlist and the current imported mods.

NOTE: it doesn't load the imports automatically and you may have to load the imports
first before proceeding.
NOTE: if you merge a modlist that doesn't depend on vanilla, like a shared modlist
you want to pack afterward you may still want to import & load vanilla while you
merge because the scriptmerger (for some reason) fails to merge only two files.
In this case, import vanilla, load the imports then merge. And after you've merged
unload the import and remove vanilla and you can safely pack your modlist.
  ";

  let mods = modlist.get_children(modlist.mods_path()).join("\n");
  let dlcs = modlist.get_children(modlist.dlcs_path()).join("\n");
  let menus = modlist.get_children(modlist.menus_path()).join("\n");

  let open_mods_help = format!("mods in the modlist:\n{}", mods);
  let open_dlcs_help = format!("dlcs in the modlist:\n{}", dlcs);
  let open_menus_help = format!("menus in the modlist:\n{}", menus);

  let content = html! {
    section {
      h1 {
        (modlist.name)
      }

      div class="row flex-center" {
        form method="post" action="/api/modlist/visibility-down" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="imported_modlist_name" value="-1";

          input type="submit" class="text-style" value="<";
        }

        span title=(visibility_level_help) {(modlist.visibility)};

        form method="post" action="/api/modlist/visibility-up" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="imported_modlist_name" value="1";

          input type="submit" class="text-style" value=">";
        }
      }

      div class="row even" {
        form method="post" action="/api/modlist/view" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="folder_name" value=".";
          input type="submit" value="open" class="text-style";
        }

        form method="post" action="/api/modlist/view" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="folder_name" value="mods";
          input type="submit" value="mods" class="text-style" title=(open_mods_help);
        }

        form method="post" action="/api/modlist/view" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="folder_name" value="dlcs";
          input type="submit" value="dlcs" class="text-style" title=(open_dlcs_help);
        }

        form method="post" action="/api/modlist/view" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="folder_name" value="menus";
          input type="submit" value="menus" class="text-style" title=(open_menus_help);
        }
      }

      div class="row even imports" {
        @if !modlist.is_packed() {
          form method="post" action="/api/modlist/merge" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="submit" value="merge" class="text-style" title=(merge_help);
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

  let view = components::page(&format!("{} - modlist", modlist_name), &content);
  
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