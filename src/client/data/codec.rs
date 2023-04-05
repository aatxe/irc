//! If the `essentials` feature is activated, we want the irc-interface trait to imply [`From<irc_proto::Command>`], so we wrap it into a trait of the same name.
//! If `essentials` is not activated, we just re-import the trait.
#[cfg(feature = "essentials")]
/// Helper trait to feature gate the [`From<crate::proto::Command>`] dependency (which is currently required anything other than barebones mode).
pub trait InternalIrcMessageOutgoing:
    irc_interface::InternalIrcMessageOutgoing + From<crate::proto::Command>
{
}

#[cfg(feature = "essentials")]
impl<T> InternalIrcMessageOutgoing for T where
    T: From<crate::proto::Command> + irc_interface::InternalIrcMessageOutgoing
{
}

#[cfg(not(feature = "essentials"))]
/// Helper trait to feature gate the [`From<crate::proto::Command>`] dependency.
pub(crate) use irc_interface::InternalIrcMessageOutgoing;

#[cfg(feature = "essentials")]
/// Helper trait to feature gate the [`std::borrow::Borrow<crate::proto::Message>`] dependency.
pub trait InternalIrcMessageIncoming:
    std::borrow::Borrow<crate::proto::Message> + irc_interface::InternalIrcMessageIncoming
{
}

#[cfg(feature = "essentials")]
impl<T> InternalIrcMessageIncoming for T where
    T: std::borrow::Borrow<crate::proto::Message> + irc_interface::InternalIrcMessageIncoming
{
}

#[cfg(not(feature = "essentials"))]
/// Helper trait to feature gate the [`std::borrow::Borrow<crate::proto::Message>`] dependency.
pub(crate) use irc_interface::InternalIrcMessageIncoming;
