use maud::{html, Markup};

pub fn header(page_title: &str) -> Markup {
  // let css_path = format!("/static/{}.css", page_title);

  html! {
    head {
      meta charset="utf-8";
      meta name="viewport" content="width=device-width, initial-scale=1.0";
      meta http-equiv="X-UA-Compatible" content="ie=edge";

      // the style is hardcoded because it allows me to ship a single binary
      // without any other files around it.
      style type="text/css" {
        (master_css_content())
      }
  
      title { (page_title) }
    }
  }
}

fn master_css_content() -> String {
  "
  html, body {
    /*font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;*/
    font-family: Bahnschrift;
    color: white;
    background: #171413;
    min-height: 100vh;
  
    padding: 0;
    margin: 0;
    font-size: 120%;
  }
  
  :root {
    --var-color-accent: #a78445;
  }

  .menu {
    display: flex;
    align-items: flex-end;
    justify-content: center;
  }
  
  .menu, #content {
    padding: .5em
  }
  
  .menu a {
    font-size: 3em;
    font-family: monospace;
    color: var(--var-color-accent);
    font-weight: bold;
  }
  
  a, .accent {
    color: var(--var-color-accent);
    text-decoration: none;
  }

  input, select {
    background: #2a2731;
    border-radius: 3px;
    border: solid 1px rgb(167 132 69 / 13%);
    padding: 4px;
    color: whitesmoke;
    font-family: Bahnschrift;
  }

  input[type='submit'] {
    background: none;
    color: grey;
    outline: none;
    border: none;
    border-radius: 0;
    cursor: pointer;
    padding: 3px;
  }

  input[type='submit']:hover {
    color: white;
  }
  
  .row {
    display: flex;
    flex-direction: row;
  }
  
  .row.even {
    justify-content: space-evenly;
  }

  .center {
    text-align: center;
    justify-content: center;
  }
  
  h1 {
    text-decoration: underline;
    text-transform: uppercase;
    text-align: center;
  }
  
  h2 {
    font-size: 1.1em;
  }
  
  .row + form {
    margin: 1em;
  }
  
  
  
  
  li input {
    margin-left: 1em;
    cursor: pointer;
  }

  .rotate-90-clockwise {
    transform: rotate(90deg);
  }

  .text-style {
    background: none;
    border: none;
    color: var(--var-color-accent);
    cursor: pointer;
  }

  .text-style:disabled, .disabled {
    text-decoration: line-through;
  }

  .small {
    font-size: 0.6em;
  }
  .tad-smaller {
    font-size: 0.7em;
  }

  .big {
    font-size: 1.6em;
  }

  .flex-end {
    justify-content: flex-end;
  }

  .flex-center {
    justify-content: center;
  }

  .disabled-folder {
    font-style: italic;
    opacity: 0.5;
  }

  .hidden {
    display: none;
  }

  .folder-display {
  }

  .folder-display + a {
    transition: 0.25s all;
    
    transform: translate(-5px, 0);
    opacity: 0;
    padding-left: .2em;
    padding-right: .6em;
  }

  .folder-display:hover + a,
  .folder-display + a:hover {
    
    padding-left: .6em;
    padding-right: .2em;
    opacity: 1;
  }
  ".to_owned()
}