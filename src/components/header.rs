use maud::{html, Markup};

pub fn header(page_title: &str) -> Markup {
  let css_path = format!("/static/{}.css", page_title);

  html! {
    head {
      meta charset="utf-8";
      meta name="viewport" content="width=device-width, initial-scale=1.0";
      meta http-equiv="X-UA-Compatible" content="ie=edge";
  
      // used for debugging
      // link href="/static/master.css" rel="stylesheet";
      // link href=(css_path) rel="stylesheet";

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
    display: flex;
    flex-direction: column;
    background:#12100f;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
  
    padding: 0;
    margin: 0;
  }
  
  :root {
    --var-color-accent: #a78445;
  }
  
  .menu {
    padding: 1em
  }
  
  .menu a {
    font-size: 3em;
    font-family: monospace;
    color: var(--var-color-accent);
    font-weight: bold;
  }
  
  a {
    color: var(--var-color-accent);
  }
  
  #content {
    max-width: 450px;
    /* background:#1d1d1d; */
    color: white;
  
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    padding: 1em;
    border-radius: 2px;
    box-shadow: 0px 6px 12px 0px rgba(0, 0, 0, 1);
    border-top: solid 6px var(--var-color-accent);
  }
  
  .row {
    display: flex;
    flex-direction: row;
  }
  
  .row.even {
    justify-content: space-evenly;
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
  
  
  ul li {
    display: flex;
    flex-direction: row;
    align-items: baseline;
    justify-content: space-between;
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
  }

  .small {
    font-size: 0.6em;
  }

  .flex-end {
    justify-content: flex-end;
  }
  ".to_owned()
}