---
delivery_guarantee: "at_least_once"
description: "The Vector `logplex` source ingests data through the Heroku Logplex HTTP Drain protocol and outputs `log` events."
event_types: ["log"]
issues_url: https://github.com/timberio/vector/issues?q=is%3Aopen+is%3Aissue+label%3A%22source%3A+logplex%22
operating_systems: ["linux","macos","windows"]
sidebar_label: "logplex|[\"log\"]"
source_url: https://github.com/timberio/vector/tree/master/src/sources/logplex.rs
status: "beta"
title: "Heroku Logplex Source"
unsupported_operating_systems: []
---

The Vector `logplex` source ingests data through the [Heroku Logplex HTTP Drain protocol][urls.logplex_protocol] and outputs [`log`][docs.data-model.log] events.

<!--
     THIS FILE IS AUTOGENERATED!

     To make changes please edit the template located at:

     website/docs/reference/sources/logplex.md.erb
-->

## Configuration

import CodeHeader from '@site/src/components/CodeHeader';

<CodeHeader fileName="vector.toml" learnMoreUrl="/docs/setup/configuration/"/ >

```toml
[sources.my_source_id]
  type = "logplex" # must be: "logplex"
  address = nil # example
```

## Options

import Fields from '@site/src/components/Fields';

import Field from '@site/src/components/Field';

<Fields filters={true}>


<Field
  common={true}
  defaultValue={null}
  enumValues={null}
  examples={[]}
  name={"address"}
  path={null}
  relevantWhen={null}
  required={true}
  templateable={false}
  type={"string"}
  unit={null}
  >

### address

The address to accept connections on.


</Field>


</Fields>

## How It Works

### Environment Variables

Environment variables are supported through all of Vector's configuration.
Simply add `${MY_ENV_VAR}` in your Vector configuration file and the variable
will be replaced before being evaluated.

You can learn more in the [Environment Variables][docs.configuration#environment-variables]
section.


[docs.configuration#environment-variables]: /docs/setup/configuration/#environment-variables
[docs.data-model.log]: /docs/about/data-model/log/
[urls.logplex_protocol]: https://github.com/heroku/logplex/blob/master/doc/README.http_drains.md