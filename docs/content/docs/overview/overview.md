+++
title = "Overview"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
top = false
+++

The Broken Spoke Projects are a collection of software that is used to automate
process around the BNA.

They are written in [Rust], [Python], and [Javascript] with [React]/[Next.js].

The infrastructure relies heavily on [Docker] containers and [Kubernetes].

## Getting started

Depending on your interesting, you may jump directly to the sub-project that
interests you. However it is most likely that you will want to start with the
infrastructure piece.

### Broken Spoke Infrastructure

The infrastructure leverages Kubernetes and Docker to run the services.

Tools and services can also be run locally, either directly from their
subdirectory, either using the minikube setup we provide (see the
[Developer Setup](./developer-setup.md) page for more details).

### BNA Core

BNA Core is the heart of the Broken spoke, but also extends to other BNA
projects. This library is written in [Rust], but also provides [Python]
bindings.

### The Spokes

The "spokes" are a set of command line utilities based off the "bnacore"
library. Each tool performs a single action. The goal is to combine them to
build pipelines that can either run locally or in the cloud.

[docker]: https://www.docker.com/
[javascript]: https://developer.mozilla.org/en-US/docs/Web/JavaScript
[kubernetes]: https://kubernetes.io/
[next.js]: https://nextjs.org/
[python]: https://www.python.org/
[react]: https://reactjs.org/
[rust]: https://www.rust-lang.org/
