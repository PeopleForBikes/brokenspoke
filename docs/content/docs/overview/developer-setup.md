+++
title = "Developer Setup"
date = 2021-11-26T08:20:00+00:00
weight = 15
template = "docs/page.html"

[extra]
toc = true
top = false
+++

The infrastructure relies heavily on [Docker] containers and [Kubernetes].

## Requirements

Before starting, it is assumed that the following tools are installed and
configured on the developer machine. If that is not the case, please refer to
the official installation instructions to prepare your environment.

- [git]
- [docker](https://www.docker.com/get-started/)
- [kubectl](https://kubernetes.io/docs/tasks/tools/)
- [minikube](https://minikube.sigs.k8s.io/docs/start/)
- [helm](https://helm.sh/docs/intro/install/)
- [just](https://github.com/casey/just#packages=)

It is also expected for all the Broken Spoke Projects to be located in the same
folder. If that is not the case, the commands described in this documentation
will have to be adjusted accordingly.

Configuring helper variables may help:

```bash
export PFB_HOME="{$HOME}/projects/PeopleForBikes"
export BROKENSPOKE_INFRA="{$PFB_HOME}/brokenspoke/infra"
```

Then make sure the directories exist and go to the right one:

```bash
mkdir -p ${PFB_HOME}
cd ${PFB_HOME}
```

### Platforms

Last but not least, while we try to be platform agnostic and provide as much
abstraction as possible, the main developers work on OSX and Linux, therefore
Windows users may have to adapt these instructions. As always, pull-requests to
help us improve the multi-platform support and automation are welcome.

## Setup the local dev environment

> **REMARK: make sure to run the commands in the correct folder.**

Start by cloning the [Broken Spoke] repository:

```bash
git clone git@github.com:PeopleForBikes/brokenspoke.git
cd ${BROKENSPOKE_INFRA}
```

Provision and configure a local Kubernetes cluster:

```bash
just
```

This will spin up [Minikube] and configure a [postgresql] instance.

The process will take a few minutes. Follow the it on the screen as it will
display some useful pieces of information.

### Connecting to the database

Export the connection string as follow:

```bash
export DATABASE_URL=postgresql://minikube:minikube@127.0.0.1:5432/bna
```

In another terminal, start the kubectl process:

```bash
kubectl port-forward --namespace default svc/postgresql 5432:5432
```

> **REMARK: this terminal will have to stay open as long as you plan to interact
> with the database.**

The database is now accessible using your favorite client, for instance [pgcli]:

```bash
pgcli $DATABASE_URL
```

or [psql]:

```bash
$ psql $DATABASE_URL -c 'SELECT COUNT(id) from city;'
 count
-------
   767
(1 row)
```

[docker]: https://www.docker.com/
[git]: https://git-scm.com/downloads
[kubernetes]: https://kubernetes.io/
[minikube]: https://minikube.sigs.k8s.io/docs/
[pgcli]: https://www.pgcli.com/
[postgresql]: https://www.postgresql.org/
[psql]: https://www.postgresql.org/docs/current/app-psql.html
