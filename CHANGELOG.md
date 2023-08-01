Recent changes (tracing-loki)
=============================

0.2.4 (2023-08-01)
------------------

- Use explicitly specified `parent` for events by @gdesmott.

0.2.3 (2023-06-16)
------------------

- Allow clean shutdown using the `BackgroundTaskController` obtained from
  `Builder::build_controller_url`. Check `examples/shutdown.rs` for an example.

0.2.2 (2023-03-08)
------------------

- Change to a builder API for configuring the logging.
- Allow specifying paths in the Loki URL by @kellerkindt.
- Allow setting HTTP headers for Loki requests. This allows setting a tenant ID
  via the `X-Scope-OrgID` header. Idea and initial implementation by
  @TheSamabo.

0.2.1 (2022-07-02)
------------------

- Allow to select reqwest backend using feature flags by @greaka.

0.2.0 (2022-05-12)
------------------

- Add span fields to the serialized events by @chrismanning
- Change level to string mapping by @juumixx (each level is now differently
  colored by Loki, previously: "debug", "informational", "notice", "warning",
  "error"; now "trace", "debug", "info", "warn", "error" like the `tracing`
  levels.
