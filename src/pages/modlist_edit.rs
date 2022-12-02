use crate::components;
use crate::models::modlist::ModList;

use actix_web::web::HttpRequest;
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

  let content = html! {
    h1 { (modlist.name) }
    h2.center { "modlist editing" }

    div.row {

      div.column {
        h3.center { "Rename" }

        form method="post" action="/api/modlist/rename" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="text" name="new_modlist_name" value=(modlist.name);
          input placeholder="New name" type="submit" value="rename";
        }

        p {
          "Rename a the modlist to a new name, replacing the original file if the destination already exists."
          br;
          "This will not work if the destination is on a different mount point
          and will break the link if the modlist was currently installed, you will have to re-install it"
        }
      }

      div.column {
        h3.center { "Delete" }

        form method="post" action="/api/modlist/delete" {
          input type="hidden" name="modlist_name" value=(modlist.name);
          input type="text" name="modlist_name_confirmation" placeholder={"type \"" (modlist.name) "\" to confirm "};
          input type="submit" value="delete";
        }

        p {
          "This will permanently delete the modlist and all of its content.
          Imported modlists and mods won't be deleted, only the links to them will."
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
