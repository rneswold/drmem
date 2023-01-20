# Tutorial

This tutorial will introduce you to some fundamental concepts of DrMem. Later, more advanced tutorials will show some powerful features that allow DrMem to control devices unattended.

For the beginner's tutorial, we'll build a very simple instance of `drmemd`. In this version, we'll use the simple backend, no external drivers, and include the GraphQL interface. This means there won't be any archiving of the device data and you don't need to purchase any commercial hardware to try things out.

This tutorial is based on v0.1.0.

## Build `drmemd`

This first step is building the executable. At the moment, we don't have pre-built executables. The main reason we haven't is because each installation has different requirements and `drmemd` is built with a set of drivers linked in[^1].

[^1]: There is an open issue to research whether we could use shared libraries for drivers and dynamically load them. Until this happens, `drmemd` will probably need to be built by the end-user.

### Building from `cargo`

You should be able to build and install the executable directly from `cargo`. To do this, you need to install Rust. Once it is installed, build and install the executable:

```shell
$ cargo install drmemd --features simple-backend,graphql
```

### Building from source

Download and build the executable:

```shell
$ git clone git@github.com:DrMemCS/drmem.git
$ cd drmem
$ cargo build --features simple-backend,graphql
```

This builds the debug version which is found at `target/debug/drmemd`. If you want to build a release version, use `cargo build --release --features simple-backend,graphql` and use the executable built at `target/release/drmemd`. Building the release takes a while, but it builds a much smaller executable.

## Configure

`drmemd` looks for configuration information in a `.drmem.toml` file. This file can be in the current directory, your home directory, or in a system-wide location. For this tutorial, we'll simply store it in the current directory.

Several small, useful drivers are always available so we'll use one of them. Create the file `.drmem.toml` with the following contents:

```toml
[[driver]]
name = "timer"
prefix = "demo-timer"
cfg = { millis = 5000, active_level = true }
```

The **name** parameter specifies which driver we're using for this instance. In this case, we're using the `timer` driver which implememts a one-shot timer. The timer driver creates two devices: `enable` and `output`. The `output` device is read-only and reflects the state of the timer. The `enable` device is readable and settable. Any time it goes from `false` to `true` transition (via settings), the timer output becomes active and stays active for the time specified by the **millis** configuration parameter.

The **prefix** parameter specifies the path to be prepended to the device names created by this driver. For this configuration, the two devices will be named `demo-timer:enable` and `demo-timer:output`. Multiple `[[driver]]` sections can use the `timer` driver to make many timer devices, but different prefixes need to be specified so all the created devices have unique names.

Each driver has a **cfg** parameter which contains configuration information specific to the driver. For timers, we specify the length of its active time using the **millis** parameter (in this example, 5 seconds.) The **active_level** parameter specifies the value of the `output` device while the timing is active.

## Run `drmemd`

Run the `drmemd` executable. It'll display some log messages as it starts up. It also opens port 3000 for GraphQL clients.

## Interacting via GraphQL

`drmemd` includes a GraphQL client so you can use a browser to interact with it. Ambitious users can write web or mobile apps with GraphQL client libraries to browse and control `drmemd` devices.

Open a browser window and go to "http://MACHINE:3000/graphiql/" -- replacing MACHINE with the name/address of the machine running DrMem. If you can't connect, you may have to open port 3000 in your firewall. If your browser successfully connects, you'll see an environment where you can submit GraphQL queries and see the results.

On the far right are two tabs, "Docs" and "Schema". Clicking on the "Docs" tab will show documentation built into DrMem which describes available queries, their arguments, and the form of the responses. Take a moment to peruse the docs.

## Drive Information

The first query we'll use is one which shows the drivers which are available in the running instance of DrMem. In the query editor, enter:

```json
query {
  driverInfo {
    name
    summary
  }
}
```

and then press the "play" button. The results pane should show a list of available drivers, including their name and a summary of what the driver does. The "docs" show that the `DriverInfo` reply also includes a "description" field. If you add that to the query, you'll get the associated description of the driver (in Markdown format, so it's not easy to read in this environment.)

## Device Information

There's also a query which returns information about the devices you defined in your configuration file. In the query editor, enter:

```json
query {
  deviceInfo {
    deviceName
    units
    settable
    driver {
      name
    }
  }
}
```

The result should be a list of two devices, both from the `timer` driver. The "deviceName" field shows the names. "units" is `null` because boolean devices typically don't have engineering units
associated with them. "settable" indicates whether you can change the device. "driver" contains information about the driver that implements the device.

The `deviceInfo` query takes two, optional arguments which filter the results of the query: `pattern` reduces the returned set to only devices whose name matches the pattern; `settable` only returns devices whose "settable" field matches the value of this argument.

## Getting Device Readings

If client applications need to access devices, they can use the `monitorDevice()` query. This query uses the GraphQL subscription service which means the query returns a stream of results until the client closes the connection. DrMem devices only return data when their value changes so a query may seem "stuck" or "hung" but as long as the connection is there, you can assume there haven't been any updates to the device.

Let's monitor the output of `demo-timer:output`. In the query panel, enter:

```json
subscription {
  monitorDevice(device:"demo-timer:output") {
    device
    stamp
    boolValue
  }
}
```

Monitoring a device always returns the current value along with the timestamp when it occurred. Then it waits for further updates. We can see these changes in the next section.

## Setting a Device

For a timer device, when the `enable` device goes from `false` to `true`, the timer starts timing.

Put the current browser window, which is monitoring the `output` device, aside and open another browser window. Connect to DrMem using the same URL as before. In this window, we're going to set the timer's `enable` device to true so it begins timing.

Enter this query:

```json
mutation {
  control {
    setDevice (name: "demo-timer:enable", value: { bool: true })
  }
}
```

When you execute this query, you'll see the timer output goes immediately to true and, after 5 seconds, goes back to false. To start the timer again, you first have to set the `enable` device to false before setting it to true.

GraphQL allows you to chain queries, but you have to add "alias" labels so results can be matched with the query. Try this:

```json
mutation {
  control {
    set_false:
      setDevice (name:"demo-timer:enable", value:{ bool:false })
    set_true:
      setDevice (name:"demo-timer:enable", value:{ bool:true })
  }
}
```

Now each time you run this double query, the timer resets; the first query sets the `enable` device to false and the second to true. The timer driver only reports *changes* to `output` so, if you reset it before it times out, you'll see `output` change to false 5 seconds later. In other words, the timer driver won't issue two true or two false values.