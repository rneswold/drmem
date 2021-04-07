// Copyright (c) 2020-2021, Richard M Neswold, Jr.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

/// Enumerates all the errors that can be reported in `drmem`. Authors
/// for new drivers or database backends should try to map their
/// errors into one of these values. If no value of `ErrorKind` is
/// appropriate, a new one could be added (requiring a new release of
/// this crate) but make sure the new code is generic enough that it
/// may be useful for other drivers or backends. For instance, don't
/// add an error value that is specific to Redis. Add a more general
/// value and use the associated description string to explain the
/// details.

#[derive(Debug,PartialEq)]
pub enum ErrorKind {
    /// Returned whenever a resource cannot be found.
    NotFound,

    /// A type mismatch is preventing the operation from continuing.
    TypeError,

    /// Returned when a communication error occurred with the backend
    /// database. Each backend will have its own recommendations on
    /// how to recover.
    DbCommunicationError,

    /// The requested operation cannot complete because the process
    /// hasn't provided proper authentication credentials.
    AuthenticationError,

    /// The requested operation couldn't complete. The description
    /// field will have more information for the user.
    OperationError,
}

#[derive(Debug,PartialEq)]
pub struct Error(pub ErrorKind, pub String);

/// This module defines the fundamental types that can be associated
/// with a device. Drivers set the type for each device they manage
/// and, for devices that can be set, only accept values of the
/// correct type.
///
/// This is the module to expand, if devices need to return new,
/// exotic types.

/// Primitive types available to devices.
#[derive(Clone, Debug, PartialEq)]
pub enum DeviceValue {
    /// Devices shouldn't use this type as it may go away. It was
    /// created to solve a limitation in an early version of `drmem`.
    Nil,

    /// For devices that return/accept a simply true/false, on/off,
    /// etc. state.
    Bool(bool),

    /// For devices that return/accept an integer value. It is stored
    /// as a signed, 64-bit value so a device returning an unsinged,
    /// 32-bit integer will have enough space to represent it.
    Int(i64),

    /// For devices that return/accept floating point numbers.
    Flt(f64),

    ///For devices that return/accept text.
    Str(String)
}

/// This trait is defined for types that can trivially be converted to
/// `types::DeviceValue`.
pub trait Compat {
    fn to_type(self) -> DeviceValue;
}

impl Compat for DeviceValue {
    fn to_type(self) -> DeviceValue {
	self
    }
}

impl Compat for &DeviceValue {
    fn to_type(self) -> DeviceValue {
	self.clone()
    }
}

impl Compat for bool {
    fn to_type(self) -> DeviceValue {
	DeviceValue::Bool(self)
    }
}

impl Compat for i64 {
    fn to_type(self) -> DeviceValue {
	DeviceValue::Int(self)
    }
}

impl Compat for f64 {
    fn to_type(self) -> DeviceValue {
	DeviceValue::Flt(self)
    }
}

impl Compat for String {
    fn to_type(self) -> DeviceValue {
	DeviceValue::Str(self)
    }
}