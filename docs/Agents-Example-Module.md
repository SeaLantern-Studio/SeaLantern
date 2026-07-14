---
module-name: {name}
update-time: {YYYY-mm-dd}
discription: {short-discription}
tag: ["{example-tag-1}","{example-tag-2}"]
---

IF ANY AGENT FIND THESE FILES(Included Example Files) ARE OUTDATE, YOU CAN UPDATE AND REPORT TO USER.

TAGS NEED 1 ~ 5.

## {Module Name}

{long-discription}

## Module Entry

- {root-folder-name}
  - {folder(e.g.:src)/}
    - {`file`}
  - {`Cargo.toml`(Optional if this module doesn't has.)}
  - {somethings}

e.g: ../docker-entry/
```
- docker-entry
  - src/
    - `main.rs`
  - `Cargo.toml`
```

DON'T COPY FROM LINE 19 ~ LINE 27, IT JUST IS AN EXAMPLE!

## {Module Info}

[{@`file-path`}]({path-link}): {discription} | {usage}
- {`function`} -> {output}: {usage}

e.g.: ../docker-entry/
```
[@`docker-entry/src/main.rs`](../docker-entry/src/main.rs): Used to run SeaLantern Http Service from Docker Env.
- `main` -> fn(sea_lantern_lib::run_headless_http()): Entry interface.
```

`file-path` value and path-link value are true file path.
`function` value is main function what is public function.

DON'T COPY FROM LINE 36 ~ LINE 45, IT JUST IS AN EXAMPLE!
