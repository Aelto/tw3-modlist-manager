use crate::components;
use crate::models::modlist::ModList;

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use maud::html;

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
      .body(view.into_string());
  }

  let modlist = some_modlist.unwrap();

  let folder_type = req.match_info().get("folder_type").unwrap_or("mods");

  let folder_name = req
    .match_info()
    .get("folder_name")
    .unwrap_or("unknown_folder");

  let all_modlists = ModList::get_all()
    .into_iter()
    .filter(|ml| modlist.name != ml.name)
    .collect::<Vec<ModList>>();

  let folder_type_singular_form = if folder_type.ends_with("s") {
    folder_type.trim_end_matches("s")
  } else {
    folder_type
  };

  let content = html! {
    h1 { (modlist.name) }
    h2.center { (folder_type_singular_form) " editing" }
    h3.center.accent.small { "[" (folder_name) "]" }

    div.row {

      div.column {
        h3.center { "Rename" }

        form method="post" action="/api/modlist/folder-rename" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="folder_type" value=(folder_type);
          input type="hidden" name="folder_name" value=(folder_name);
          input type="text" name="new_folder_name" value=(folder_name);
          input placeholder="New name" type="submit" value="rename";
        }

        p {
          "Rename a file or directory to a new name, replacing the original file if the destination already exists."
          br;
          "This will not work if the destination is on a different mount point."
        }
      }

      div.column {
        h3.center { "Move" }

        form method="post" action="/api/modlist/folder-move" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="folder_type" value=(folder_type);
          input type="hidden" name="folder_name" value=(folder_name);

          select name="new_modlist_name" {
            @for ml in &all_modlists {
              option value=(ml.name) { (ml.name) }
            }
          }

          input type="submit" value="move";
        }

        p {
          (folder_name) " is currently in the " a href={"/modlist/"(modlist_name)} { (modlist_name) } " modlist."
          br;
          "Move it to another modlist."


        }
      }

      div.column {
        h3.center { "Delete" }

        form method="post" action="/api/modlist/folder-delete" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="hidden" name="folder_type" value=(folder_type);
          input type="hidden" name="folder_name" value=(folder_name);
          input type="text" name="folder_name_confirmation" placeholder={"type \"" (folder_name) "\" to confirm "};
          input type="submit" value="delete";
        }

        p {
          "TIP: if you don't want to lose the mod you can also create yourself
          a modlist where you store your unused mods and dlcs. And instead you
          move the folder to this modlist."
        }
      }

    }

    style type="text/css" { (get_stylesheet()) }
  };

  let view = components::page(&format!("{} - modlist", modlist_name), &content);

  HttpResponse::Ok()
    .content_type("text/html")
    .body(view.into_string())
}

fn get_stylesheet() -> String {
  "
    body {
      font-size: 100%;
    }

    h2 {
      position: relative;
    }

    h2::before,
    h2::after {
      content: '';
      position: absolute;
      height: 1.5px;
      width: 150px;
      top: 50%;
      background: rgba(250, 250, 250, 0.05);
    }

    h2::before {
      right: 65%;
    }

    h2::after {
      left: 65%;
    }

    .row {
      display: flex;
      flex-wrap: wrap;
      justify-content: center;
    }

    .column {
      max-width: 250px;
      flex-grow: 1;
      padding: 0 2em;
    }
    .column + .column {
      border-left: solid 1px rgba(250, 250, 250, 0.05);
    }
    .column h3 {
      text-transform: uppercase;
    }

    form {
      display: flex;
      flex-direction: column;
    }

    form input[type='submit'] {
      font-size: 150%;
    }

  "
  .to_owned()
}
