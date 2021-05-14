use std::path::PathBuf;

use crate::components;
use crate::models::modlist::ModList;
use crate::utils::symlinks::get_children_without_symlinks;

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

  let content = html! {
    section {
      div.row.center.baseline {
        h1.modlist-name {
          (modlist.name)
        }
        
        a class="small" href={"/modlist/"(modlist.name)"/edit"} { "edit" }
      }

      div class="row center imports" {
        @if !modlist.is_packed() {
          form method="post" action="/api/modlist/merge" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="submit" value="merge" class="text-style" title=(merge_help);
          }

          div {
            button class="text-style" onclick="start_socket_merging()" title=(merge_help) { "merge scripts" };
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

      (get_merge_conflict_view())

      div class="folder-list-container" {

        div class="column tad-smaller" {
          form method="post" action="/api/modlist/view" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="hidden" name="folder_name" value="mods";
            input type="submit" value="Installed mods" class="text-style big";
          }

          (get_modlist_folders_view(&modlist, &FolderViewType::Mods, true))
        }

        div class="column tad-smaller" {
          form method="post" action="/api/modlist/view" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="hidden" name="folder_name" value="dlcs";
            input type="submit" value="Installed DLCs" class="text-style big";
          }

          (get_modlist_folders_view(&modlist, &FolderViewType::Dlcs, true))
        }

        div class="column tad-smaller" {
          form method="post" action="/api/modlist/view" {
            input type="hidden" name="modlist_name" value=(modlist.name);
            input type="hidden" name="folder_name" value="menus";
            input type="submit" value="Installed menus" class="text-style big";
          }

          (get_modlist_folders_view(&modlist, &FolderViewType::Menus, true))
        }

      }

      section class="imports" {
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
      script type="text/javascript" { (maud::PreEscaped(get_javascript())) }
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


    .folder-list-container {
      display: flex;
      justify-content: space-evenly;
    }
    .folder-list-container .column + .column {
      margin-left: 2em;
    }

    .folder-list-container .column .folder-list {
      list-style: none;
    }

    .folder-list-container .column h3, .folder-list-container ul {
      margin: 0;
      padding: 0;
    }

    .folder-list-container .column .folder-listing {
      padding-left: 1em;
      margin-left: 5px;
      border-left: solid 1px white;
      list-style: none;
    }

    .folder-list-container .folder-listing, .folder-list-container .folder-list {
      position: relative;
    }

    .folder-list-container .folder-listing:not(.do-not-hide-bar):before {
      content: '';
      position: absolute;
      border-bottom: solid 1px white;
      width: 13px;
      top: 50%;
      left: 0;
    }

    /*
    small hack to hide the center horizontal bar
    */
    .folder-listing .folder-list::before {
      content: '';
      position: absolute;
      top: 0;
      width: 13px;
      background: #171413;
      left: -49.5px;
      height: 100%;
    }

    .folder-listing h3 {
      position: relative;
    }
    .folder-listing h3::before {
      content: '';
      position: absolute;
      top: 50%;
      width: 15px;
      background: white;
      left: -50px;
      height: 1px;
    }
    

    .folder-listing:last-child .folder-list::before {
      left: -51px;
      width: 16px;
      top: 12px;
      height: calc(100% + 9px);
    }

    .folder-listing:last-child::after {
      content: '';
      background: #171413;
      width: 5px;
      height: 50%;
      position: absolute;
      top: calc(50% + 1px);
      left: -1px;
    }

    .toggle-modlist-folder-button {
      display: flex;
      align-items: baseline;
      background: #171413;
      z-index: 1;
      padding: 0 .5em;
      cursor: pointer;
    }

    section.imports {
      margin: auto;
      margin-top: 3em;
      max-width: 350px;
    }
    section.imports ul {
      list-style: none;
    }
    section.imports ul li {
      display: flex;
      justify-content: space-between;
    }

    .modlist-name + a {
      transition: 0.25s all;
      
      transform: translate(-5px, 0);
      opacity: 0;
      padding-left: .2em;
      padding-right: .6em;
    }
  
    .modlist-name:hover + a,
    .modlist-name + a:hover {
      
      padding-left: .6em;
      padding-right: .2em;
      opacity: 1;
    }

    .merge-conflict-view {
      margin: 5em 0;
      font-size: 12px;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;
    }

    .merge-conflict-view .merge-conflict {
      margin: 4em 0;
    }

    .merge-conflict > div {
      display: flex;
    }

    .merge-conflict .row .left {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 50px;
      position: relative;
    }

    .merge-conflict .row .left .modname {
      position: absolute;
      left: 0;
      transform: translate(-100%, 0%);
    }

    .merge-conflict .row .left button {
      background: none;
      outline: none;
      border: none;
      cursor: pointer;
      color: green;
    }

    .merge-conflict .row pre {
      flex-grow: 1;
      cursor: default;
    }

    

    .merge-conflict .row.accepted .reject {
      display: none;
    }

    .merge-conflict .row.rejected .accept {
      display: none;
    }

    .merge-conflict .row .left:hover + pre {
      background: rgba(250, 250, 250, .02);
    }

    .merge-conflict button {
      transition: .075s all ease-in-out;
    }
    .merge-conflict button:hover {
      transform: scale(1.25);
    }
    .merge-conflict button:active {
      transform: scale(2);
    }

    .merge-conflict .row.rejected pre {
      opacity: .1;
    }

    pre {
      margin: 0;
    }

    .context-before, .context-after {
      opacity: .1;
    }

    .ours {
      color: #8bc34a;
    }

    .theirs {
      color: #E91E63;
    }
  ".to_owned()
}

enum FolderViewType {
  Mods,
  Dlcs,
  Menus
}

fn get_folder_from_view_type(modlist: &ModList, view_type: &FolderViewType) -> PathBuf {
  match view_type {
    FolderViewType::Mods => modlist.mods_path(),
    FolderViewType::Dlcs => modlist.dlcs_path(),
    FolderViewType::Menus => modlist.menus_path(),
  }
}

fn get_modlist_folders_view(modlist: &ModList, view_type: &FolderViewType, is_top_level: bool) -> maud::Markup {
  let is_modlist_packed = modlist.is_packed();

  // when a modlist is packed it doesn't show all the mods but only the pack folder
  if !is_top_level {
    match (&view_type, is_modlist_packed) {
      (FolderViewType::Mods, true) => {
        return html! {
          ul class="folder-list" {
            li {
              h3.packed-folder { (components::modlist_link(&modlist.name)) span.small{" packed"} }
      
              ul class={"folder-list " (if !is_top_level {"hidden"} else {""})} {
                li.folder-listing {
                  (&modlist.packed_folder_name())
                }
              }
            }
          }
        }
      }
  
      _ => {}
    };
  }

  let folder_root = get_folder_from_view_type(modlist, &view_type);
  let children_result = get_children_without_symlinks(&folder_root);

  if let Err(error) = children_result {
    return html! {
      li {
        "An error occured when fetching the modlist " (modlist.name) " folders. ERROR:" (error)
      }
    }
  }

  let children = children_result.unwrap();

  let mut imported_mods = Vec::new();
  for import in &modlist.imported_modlists {
    let some_modlist = ModList::get_by_name(import);

    if some_modlist.is_none() {
      continue;
    }

    let mut imported_modlist = some_modlist.unwrap();
    if let Err(error) = imported_modlist.read_metadata_from_disk() {
      let error_html = html! {
        ul class="folder-list" {
          li class="folder-listing" {
            h3 { (&imported_modlist.name) }

            ul class="folder-list" {
              li class="folder-listing" { "Could not load the metadata from the modlist" (components::modlist_link(&imported_modlist.name)) }
              li class="folder-listing" { "ERROR: " (&error) }
            }
          }
        }
      };

      imported_mods.push((imported_modlist, error_html));
      
      continue;
    }

    imported_mods.push((ModList::get_by_name(&imported_modlist.name).unwrap(), get_modlist_folders_view(&imported_modlist, view_type, false)));
  }


  html! {
    ul class="folder-list" {
      li {
        h3 {
          (components::modlist_link(&modlist.name))
        }

        ul class={"folder-list " (if !is_top_level {"hidden"} else {""})} {
          @match view_type {
            FolderViewType::Mods => {
              @for child in children {
                li class="folder-listing" { (components::mod_display(&child, &modlist.name, is_modlist_packed)) }
              }
            },

            FolderViewType::Dlcs => {
              @for child in children {
                li class="folder-listing" { (components::dlc_display(&child, &modlist.name)) }
              }
            }

            FolderViewType::Menus => {
              @for child in children {
                li class="folder-listing" { (components::menu_display(&child, &modlist.name)) }
              }
            }
          }
          
      
          @for (_imported_modlist, imported_html) in imported_mods {
            li class={"folder-listing row do-not-hide-bar" } {
              span.toggle-modlist-folder-button { "[+]" }
              (imported_html)
            }
          }
        }
      }
    }
  }
}

fn get_merge_conflict_view() -> maud::Markup {
  html! {
    div class="hidden merge-conflict-view" {
      h2.filename.center {}
      div.conflict-count.center {}

      div class="merge-conflict hidden" {
        div class="row context-before" {
          div class="left" {
          }
          pre contenteditable="true" {}
        }

        div class="row content original rejected" {
          div class="left" {
            button.reject { "❌" }
            button.accept { "✔" }

            div.modname { "original" }
          }
          pre contenteditable="true" {}
        }
        div class="row content ours accepted" {
          div class="left" {
            button.reject { "❌" }
            button.accept { "✔" }

            div.modname { "mergedfiles" }
          }
          pre contenteditable="true" {}
        }
        div class="row content theirs accepted" {
          div class="left" {
            button.reject { "❌" }
            button.accept { "✔" }

            div.modname { }
          }
          pre contenteditable="true" {}
        }

        div class="row context-after" {
          div class="left" {
          }
          pre contenteditable="true" {}
        }
      }
    
      div.actions.row.hidden {
        button.resolve.text-style { "resolve" }
        button.abandon.text-style { "abandon" }
      }

    }
  }
}

fn get_javascript() -> String {
  "
  window.addEventListener('click', e => {
    if (e.target.matches('.folder-list .toggle-modlist-folder-button')) {
      const folder_list = e.target.nextElementSibling.querySelector('.folder-list');
  
      e.target.parentElement.classList.toggle('do-not-hide-bar');
      folder_list.classList.toggle('hidden');
      e.target.textContent = e.target.textContent
        .replace('-', '$')
        .replace('+', '-')
        .replace('$', '+');
    }
  });

  function openwebsocket() {
    let socket = null;
    try {
      socket = new WebSocket('ws://localhost:5001', 'rust-websocket');
    } catch (exception) {
        console.error(exception);
    }

    socket.onerror = function(error) {
        console.error(error);
    };

    socket.onopen = function(event) {
        this.onclose = function(event) {};

        this.onmessage = function(event) {
          const data = JSON.parse(event.data);

          if (!data.conflicts.length) {
            Array.from(document.querySelectorAll('.merge-conflict.custom'))
            .forEach(node => node.remove());

            socket.close();

            return;
          }

          show_conflicts(data, socket);
        };

        // this.send('start');
    };
  }

  function show_conflicts(conflict_data, socket) {
    console.log(conflict_data);

    document.querySelector('.merge-conflict-view .actions .resolve').onclick = e => {
      const conflict_resolvers = Array.from(document.querySelectorAll('.merge-conflict.custom'));

      for (let i = 0; i < conflict_resolvers.length; i += 1) {
        const $resolver = conflict_resolvers[i];


        const $context_before = $resolver.querySelector('.context-before');
        conflict_data.conflicts[i].context_before = $context_before.querySelector('pre').textContent;

        const $context_after = $resolver.querySelector('.context-after');
        conflict_data.conflicts[i].context_after = $context_after.querySelector('pre').textContent;

        const $original = $resolver.querySelector('.original');
        if ($original.classList.contains('accepted') && !$original.classList.contains('rejected')) {
          conflict_data.conflicts[i].original = $original.querySelector('pre').textContent;
        }
        else {
          conflict_data.conflicts[i].original = '';
        }

        const $ours = $resolver.querySelector('.ours');
        if ($ours.classList.contains('accepted') && !$ours.classList.contains('rejected')) {
          conflict_data.conflicts[i].ours = $ours.querySelector('pre').textContent;
        }
        else {
          conflict_data.conflicts[i].ours = '';
        }

        const $theirs = $resolver.querySelector('.theirs');
        if ($theirs.classList.contains('accepted') && !$theirs.classList.contains('rejected')) {
          conflict_data.conflicts[i].theirs = $theirs.querySelector('pre').textContent;
        }
        else {
          conflict_data.conflicts[i].theirs = '';
        }
      }

      const message = JSON.stringify(conflict_data);
      socket.send(message);

      // remove all the conflicts that were previously shown
      Array.from(document.querySelectorAll('.merge-conflict.custom'))
        .forEach(node => node.classList.add('to-recycle'));

      document.querySelector('.actions.row').classList.add('hidden');
      document.querySelector('.merge-conflict-view .filename').textContent = '';
      document.querySelector('.merge-conflict-view .conflict-count').textContent = '';
    };

    document.querySelector('.merge-conflict-view .filename').textContent = conflict_data.file_name;
    document.querySelector('.merge-conflict-view .conflict-count').textContent = `${conflict_data.conflicts.length} conflicts`;

    document.querySelector('.actions.row').classList.remove('hidden');

    // first we remove all the conflicts that were previously shown
    Array.from(document.querySelectorAll('.merge-conflict.custom'))
      .forEach(node => node.classList.add('to-recycle'));

    // this is the base node, we copy it for every conflict we have
    const $mergeconflict = document.querySelector('.merge-conflict');

    for (const conflict of conflict_data.conflicts) {
      // first we check if there is a conflict node to recycle
      let $custom = document.querySelector('.to-recycle');

      const recycled = $custom !== null;
      if (!recycled) {
        $custom = $mergeconflict.cloneNode(true);
        $custom.classList.remove('hidden');
        $custom.classList.add('custom');
      }
      else {
        $custom.classList.remove('to-recycle');
      }


      $custom.querySelector('.context-before pre').textContent = conflict.context_before;
      $custom.querySelector('.context-after pre').textContent = conflict.context_after;
      $custom.querySelector('.content.original pre').textContent = conflict.original;
      $custom.querySelector('.content.ours pre').textContent = conflict.ours;
      $custom.querySelector('.content.theirs pre').textContent = conflict.theirs;

      $custom.querySelector('.content.original').classList.remove('accepted');
      $custom.querySelector('.content.ours').classList.remove('rejected');
      $custom.querySelector('.content.theirs').classList.remove('rejected');
      $custom.querySelector('.content.original').classList.add('rejected');
      $custom.querySelector('.content.ours').classList.add('accepted');
      $custom.querySelector('.content.theirs').classList.add('accepted');

      $custom.querySelector('.content.theirs .modname').textContent = conflict_data.mod_name;

      if (!recycled) {
        $custom.addEventListener('click', e => {
          if (e.target.matches('button')) {
            const row = e.target.parentElement.parentElement;
            row.classList.toggle('accepted');
            row.classList.toggle('rejected');
          }
        })

        $mergeconflict.parentElement.appendChild($custom);
      }
    }

    // first we remove all the conflicts that were previously shown
    Array.from(document.querySelectorAll('.to-recycle'))
      .forEach(node => node.remove());
  }

  function start_socket_merging() {
    const modlist_name = location.pathname.split('/').slice(-1)[0];

    fetch('/api/modlist/merge', {
      method: 'POST',
      body: `modlist_name=${modlist_name}`,
      headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
      },
      method: 'post',
    })
    .then(console.log)
    .catch(console.log);
  
    setTimeout(() => {
      openwebsocket();
    }, 50);
  }
  
  ".to_string()
}