# ip_geo

IP geolocation, designed for [Grafana](https://grafana.com/oss/grafana/) [Geomap](https://grafana.com/docs/grafana/latest/panels-visualizations/visualizations/geomap/).

ip_geo is currently incomplete and highly experimental.
It will follow semantic versioning after its first release,
but for now it will morph and change without appropriate versioning.
For a more optimized, mature, and accurate (but still open source) solution, see [IPFire Location](https://www.ipfire.org/location).
In newer versions of Tor, the IP geolocation database that this project relies on is actually extracted from IPFire's location database.

## Project

This directory only includes the library that parses and searches [Tor IP geolocation databases](https://packages.ubuntu.com/noble/tor-geoipdb).

### [`cli/`](./cli/)

Contains a command line utility for resolving IP addresses to countries.

### [`geo/`](./geo/)

A Crate for generating a list of country codes and names
based on data sourced from [`location(8)`](https://www.ipfire.org/location/how-to-use/cli).\
Used to generate [`src/country_list.rs`](./src/country_list.rs).

Depends on having `location(8)` in `$PATH`
such that it can be run with `cmd /C location` (on Windows) or `sh -c location` (otherwise).

### [`server/`](./server/)

Contains a HTTP API for resolving IP addresses to countries.

## License

ip_geo is licensed under the GNU Affero General Public License version 3, or (at your option) any later version.
You should have received a copy of the GNU Affero General Public License along with ip_geo, found in [LICENSE](./LICENSE).
If not, see \<[https://www.gnu.org/licenses/](https://www.gnu.org/licenses/)>.
