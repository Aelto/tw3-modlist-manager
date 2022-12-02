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

  let mut modlist = some_modlist.unwrap();

  if let Err(error) = modlist.read_metadata_from_disk() {
    let content = html! {
      h1 { "Could not read modlist metadata" }
      p { (error) }
    };
    let view = components::page("root", &content);

    return HttpResponse::Ok()
      .content_type("text/html")
      .body(view.into_string());
  }

  let content = html! {
    section {
      (get_merge_conflict_view())

      div.merge-started { "merging..." }
      div.merge-finished.hidden { "merge finished" }


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

    .menu {
      justify-content: unset;
    }

    .menu a {
      display: none;
    }

    #content, menu {
      padding: 0;
    }

    .merge-started,
    .merge-finished {
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      transition: 1s all ease-in-out;
      font-size: 2em;
      display: block;
    }

    .merge-finished.hidden {
      transform: translate(-50%, 60%);
      opacity: 0;
    }

    .merge-started {
      animation: 1s ease-out 0s infinite wobble;
    }

    @keyframes wobble {
      0% {
        transform: translate(-50%, -50%) scale(1);
      }

      30% {
        transform: translate(-50%, -50%) scale(1);
      }
      35% {
        transform: translate(-50%, -50%) scale(1.2);
      }
      40% {
        transform: translate(-50%, -50%) translateY(-50%, 0);
      }
    }

    .merge-started.hidden {
      display: none;
    }

    button.resolve {
      position: fixed;
      font-size: 2.5em;
      left: 50%;
      top: 1em;
      transform: translate(-50%, 0);
      background: #171413;
      padding: .1em 2em;
      border: solid 1px var(--var-color-accent);
      z-index: 10;

      transition: .05s transform;
    }

    button.resolve.hidden {
      display: block;
      transform: translate(-50%, -300px);
    }

    button.resolve:hover {
      color: #d6b06c;
      border-color: #d6b06c;
    }

    button.resolve:active {
      transform: translate(-50%, 0) scale(1.05);
    }

    button.resolve:not(.hidden) {
      animation: .25s ease-out 0s 1 slideInFromTop;
    }

    @keyframes slideInFromTop {
      0% {
        transform: translate(-50%, -100%);
      }
      100% {
        transform: translateY(-50%, 0);
      }
    }

    .merge-conflict-view {
      font-size: 12px;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
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

    .merge-conflict .row .left button.active {
      outline: solid 1px white;
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
      font-family: inherit;
      font-size: 1.5em;
    }

    .context-before, .context-after {
      opacity: .1;
    }

    .ours {
      color: grey;
    }

    .theirs {
      color: var(--var-color-accent);
    }
  "
  .to_owned()
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

      button.resolve.hidden.text-style { "resolve" }
      // div.actions.row.hidden {
        // "-"
        // button.abandon.text-style { "abandon" }
      // }

    }
  }
}

fn get_javascript() -> String {
  "
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
            socket.close();

            return finish_socket_merging();
          }

          show_conflicts(data, socket);
        };

        // this.send('start');
    };
  }

  function show_conflicts(conflict_data, socket) {
    console.log(conflict_data);

    document.querySelector('.merge-started').classList.add('hidden');

    document.querySelector('.merge-conflict-view button.resolve').onclick = e => {
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

      document.querySelector('button.resolve').classList.add('hidden');
      // document.querySelector('.merge-conflict-view .filename').textContent = '';
      // document.querySelector('.merge-conflict-view .conflict-count').textContent = '';
    };

    document.querySelector('.merge-conflict-view .filename').textContent = conflict_data.file_name;
    document.querySelector('.merge-conflict-view .conflict-count').textContent = `${conflict_data.conflicts.length} conflicts`;

    document.querySelector('button.resolve').classList.remove('hidden');

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

    // we set all buttons as inactive
    Array.from(document.querySelectorAll('button.active'))
      .forEach(node => node.classList.remove('active'));

    // setTimeout(() => {
      document.querySelector('.merge-conflict.custom .ours').scrollIntoView({
        behavior: 'smooth',
        block: 'center',
        inline: 'center'
      });
    // }, 50);
  }

  function start_socket_merging() {
    const modlist_name = location.pathname.split('/').slice(-2)[0];

    fetch('/api/modlist/merge-scripts', {
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

  function finish_socket_merging() {
    document.querySelector('.merge-finished').classList.remove('hidden');

    Array.from(document.querySelectorAll('.merge-conflict.custom'))
      .forEach(node => node.remove());

    document.querySelector('.merge-conflict-view .filename').textContent = '';
    document.querySelector('.merge-conflict-view .conflict-count').textContent = '';

    setTimeout(() => {
      // remove the last portion of the url
      location.href = location.href.split('/').slice(0, -1).join('/');
    }, 1500);
  }

  document.addEventListener('keypress', e => {
    if (e.target === document.body && e.keyCode === 32) { // space
      document.querySelector('button.resolve').click();
    }

    const z = 122;
    const w = 119;
    const q = 113;
    const s = 115;
    const d = 100;
    if (e.keyCode >= d || e.keyCode <= z) {
      const buttons = Array.from(document.querySelectorAll('.merge-conflict.custom .rejected button.reject, .merge-conflict.custom .accepted button.accept'));

      if (e.keyCode === s) {
        const active_button_index = buttons.reduce((acc, el, i) => el.classList.contains('active', 0) ? i : acc, -1);
  
        const next_active_button = active_button_index + 1;
  
        if (next_active_button < buttons.length) {
          buttons[next_active_button].classList.add('active');
          buttons[active_button_index].classList.remove('active');
          buttons[next_active_button].scrollIntoView({
            behavior: 'smooth',
            block: 'center',
            inline: 'center'
          });
        }
      }

      if (e.keyCode === z || e.keyCode === w) {
        const active_button_index = buttons.reduce((acc, el, i) => el.classList.contains('active', 0) ? i : acc, -1);
  
        const next_active_button = active_button_index - 1;
  
        if (next_active_button >= 0) {
          buttons[next_active_button].classList.add('active');
          buttons[active_button_index].classList.remove('active');
          buttons[next_active_button].scrollIntoView({
            behavior: 'smooth',
            block: 'center',
            inline: 'center'
          });
        }
      }

      if (e.keyCode === d) {
        const button = document.querySelector('.merge-conflict.custom .rejected button.reject.active');

        if (button !== null) {
          button.click();
          button.classList.remove('active');
          button.parentElement.querySelector('button.accept').classList.add('active');
        }
      }

      if (e.keyCode === q) {
        const button = document.querySelector('.merge-conflict.custom .accepted button.accept.active');

        if (button !== null) {
          button.click();
          button.classList.remove('active');
          button.parentElement.querySelector('button.reject').classList.add('active');
        }
      }
    }

    console.log(e.keyCode);
  });

  // start it automatically when the page is loaded
  start_socket_merging();
  
  ".to_string()
}
