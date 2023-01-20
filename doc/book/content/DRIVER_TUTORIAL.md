# Writing Device Drivers

For this tutorial, we'll recreate the timer driver, which is alwats included in the DrMem executable. This tutorial is based on the 0.2.0 API.

## Planning

A timer is one of those fundamental building blocks that you'd use to assemble more complex behavior. The purpose of a timer is to wait for a trigger before becoming active for a length of time. This means we need a device to represent the state of the timer. We'll call it `output` and make it a boolean device, where `false` means the timer is inactive and `true` when active. To trigger the timer, we'll define a settable, boolean device called `enable`. 

It's important to plan out how input signals are to control the state of the device. For a timer, should setting the `enable` device to `true` start timing? What if the timer is being controlled by a periodic signal? Do we want multiple `true` values to reset the timer?

For the internal timer driver, it was decided that the trigger input needs to go from `false` to `true` in order to start the timer. It, while timing, `enable` goes `false` and back to `true`, the timer will reset and start a new timing cycle. If a timer's cycle gets extended in this way, the `output` device won't record a new reading.

We could make the length of the timer another settable device, but that starts to complicate the logic and doesn't seem to be the common use-case. So, instead, the length of the timer will be constant and given as a configuration parameter.

### Common Devices

It you're writing a driver for a common class of hardware, please follow the set of device names and types used by other drivers. For instance, if you're adding support for a company's LED bulb, please use the common names used by other LED bulb drivers. This way all LED bulbs can be used the same way.

## Create Driver Module



## Implement the Driver

A driver is an async task, so it is able to do anything an async task can do. That being said, drivers should keep CPU usage to a minimum. If your driver needs other subtasks to help do its job, you should write unit tests to make sure these other resources are freed properly. Despite this flexibility, a driver needs to implement the `Driver` trait and uses communication channels with core to report device updates and to receive device settings.

### Initialization
### Running

--------
- Determine set of devices to be created by an instance of the driver
  - Should the set of devices match a list of standardized devices for the target hardware?
  - What are the types of the devices?
  - Which devices are settable?
  - How much history do you want to save?

- Create async task that implements the `Driver` trait.

- The `create()` method is called once to initialize the driver.
  - Set up persistent resources for the instance
  - Register the devices that are controlled by the instance

- The `run()` method gets called and isn't expected to return.
  - Driver can do practically anything -- it's an `async` task
  - Typically it sets up a `loop {}` with a use of the `tokio::select` macro to wait for one of several future to complete.
