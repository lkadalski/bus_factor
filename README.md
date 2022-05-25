# Bus Factor

## About Bus Factor
This is simple bus factor application which main aim is to fetch popular GitHub's projects with a bus factor of 1.
Program accepts two parameters `language` and `project_count`.
Program fetches the first `project_count` most popular projects from the given `language`.

## Usage
Before running please setup GITHUB_ACCESS_TOKEN as a env variable.

Typical use:<br>
`$ bus_factor --language rust --project_count 10` <br>
or <br>
`$ cargo run -- --language rust --project_count 10` <br>
or with extra logs: <br>
`$ RUST_LOG=DEBUG bus_factor --language rust --project_count 10` <br>
Possible values for `RUST_LOG` is `INFO, WARN, ERROR, DEBUG, TRACE` 


