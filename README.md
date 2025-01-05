<h1 align="center"> ━━━━  ❖  ━━━━ </h1>

<!-- BADGES -->
<div align="center">
   <p></p>

<img src="https://img.shields.io/github/stars/dotzenith/tok?color=F8BD96&labelColor=302D41&style=for-the-badge">

<img src="https://img.shields.io/github/forks/dotzenith/tok?color=DDB6F2&labelColor=302D41&style=for-the-badge">

<img src="https://img.shields.io/github/repo-size/dotzenith/tok?color=ABE9B3&labelColor=302D41&style=for-the-badge">

<img src="https://img.shields.io/github/commit-activity/y/dotzenith/tok?color=96CDFB&labelColor=302D41&style=for-the-badge&label=COMMITS"/>
   <br>
</div>

<p/>

---

## ❖ Tok

Tok is a commandline client for [TickTick](https://ticktick.com/)

---

## ❖ Installation

#### Shell

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/dotzenith/tok/releases/latest/download/tok-installer.sh | sh
```

#### Brew

```sh
brew install dotzenith/tap/tok
```

#### Powershell

```sh
powershell -ExecutionPolicy ByPass -c "irm https://github.com/dotzenith/tok/releases/latest/download/tok-installer.ps1 | iex"
```

#### Cargo

```sh
cargo install tok
```

#### Binaries

Pre-Compiled binaries for linux, mac, and windows are available in
[Releases](https://github.com/dotzenith/tok/releases)

#### Source

- First, install [rust](https://rustup.rs/)

```sh
git clone https://github.com/dotzenith/tok.git
cd tok
cargo build --release
./target/release/tok
```

---

## ❖ Requirements

1. Create a new app in the [TickTick Developer Center](https://developer.ticktick.com/manage)
2. Set the OAuth redirect URL (NOT App Service URL) to something like: "http://127.0.0.1:8000/"
3. Set environment variables as follows:
```sh
export TICKTICK_CLIENT_ID='client_id'
export TICKTICK_CLIENT_SECRET='client_secret'
export TICKTICK_REDIRECT_URL='http://127.0.0.1:8000/'
```
> Note the single quotes. Double quotes will cause trouble

---

## ❖ Usage

```
A CLI client for tick tick

Usage: tok <COMMAND>

Commands:
  show      Show To-Do items accross projects
  complete  Complete a given To-Do item accross projects
  delete    Delete a given To-Do item accross projects
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### ❖ Subcommands

**Every** subcommand allows showing tasks from different time frames. They are as follows:

Example:
```sh
$ tok show today             # shows tasks with a due date for today (includes overdue)
$ tok complete tomorrow      # shows tasks with a due date for tomorrow
$ tok delete week            # shows tasks with a due date within the next 7 days (includes overdue)
$ tok show all               # shows all tasks
```

### ❖ Filter by project name

**Every** subcommand allows filtering by project name

```sh
$ tok show today --project Cooking      # Shows tasks due today in the "Cooking" project
```
> Note: Project names are case-sensitive and must match exactly

---

## ❖ Limitations

[TickTick Developer Docs](https://developer.ticktick.com/docs) can be found here.

I started working on this with a lot of enthusiasm because I thought at long last, I had found
a solution to all of my woes. I've long wanted a commandline client for a To-Do app that works on
all of my devices. TickTick has an app for all of my devices and I was **elated** to find out there
was a documented API for it.

A few weeks later, and I now wish that I had never discovered this API in the first place. I believe
I would've been happier without it.

Here's a non-exhaustive list of limitations for this client:

- Cannot fetch a Task if it is not associated with a Project. The API simply has no way to facilitate this.
- Cannot create Tasks. This is because the API does not allow setting a project for a new task. So while
I could have spent time adding Task creation ability, there would simply be no way to see the Task you just created.
- Subtasks are not supported. This one is not a fault of API. I just don't use the feature and did not think it
worthwhile to spend any amount of time on it.
- The Auth workflow might randomly not work. Almost as if the API flips a coin when it comes to Auth. Sometimes
it might say "credentials are invalid" (they are), or it might error out when exchanging the Auth Code for an
access token. Just try a few times. It'll work.
- Rate limiting happens rather frequently. This is because we have to make **Multiple** requests every time we fetch
tasks. Once to fetch all the projects, with one request **each** for every project. This also means you might get
limited unless the `--project` flag is passed in, which naturally only has to fetch one project's data.

Enough complaining. This was built for my personal usage and I will stick with it until my subscription expires.
Will look for a different To-Do app after that.

I hope the irony of struggling to find a developer focused To-Do app when every developer builds a To-Do app as a first
project is not lost on the reader.

---

## ❖ What's New?

0.1.0 - Initial release

---

<div align="center">

<img src="https://img.shields.io/static/v1.svg?label=License&message=MIT&color=F5E0DC&labelColor=302D41&style=for-the-badge">

</div>
