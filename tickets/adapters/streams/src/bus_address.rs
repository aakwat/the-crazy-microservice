//! `BusAddress` — where the event bus is and which ACL user tickets logs in as.

use domain::PublishError;
use redis::{ConnectionAddr, ConnectionInfo, IntoConnectionInfo, RedisConnectionInfo};

use crate::publish_error;

pub struct BusAddress {
  pub host: String,
  pub port: u16,
  pub user: String,
  pub password: String,
}

impl BusAddress {
  /// Typed rather than a URL, so a password with URL characters still works.
  pub(crate) fn connection_info(self) -> Result<ConnectionInfo, PublishError> {
    let login = RedisConnectionInfo::default()
      .set_username(self.user)
      .set_password(self.password);
    let info = ConnectionAddr::Tcp(self.host, self.port)
      .into_connection_info()
      .map_err(publish_error)?;
    Ok(info.set_redis_settings(login))
  }
}
