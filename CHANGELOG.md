Recent changes (tracing-loki)
=============================

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
