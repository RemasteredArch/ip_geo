# IP Geo

IP geolocation, designed for [Grafana](https://grafana.com/oss/grafana/) [Geomap](https://grafana.com/docs/grafana/latest/panels-visualizations/visualizations/geomap/).

This directory only includes the library that parses and searches the [Tor IP geolocation databases](https://packages.ubuntu.com/noble/tor-geoipdb).\
See [`cli/`](./cli/) for the actual command line utility and server.

[`geo/`](./geo/) is a Crate for deserializing country codes based on data sourced from Wikidata.

# License

IP Geo is licensed under the GNU Affero General Public License version 3, or (at your option) any later version.
You should have received a copy of the GNU Affero General Public License along with IP Geo, found in [LICENSE](./LICENSE).
If not, see \<[https://www.gnu.org/licenses/](https://www.gnu.org/licenses/)>.
