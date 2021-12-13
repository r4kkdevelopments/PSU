using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace psu_rebirth.DataTypes.Reflection {
    public class Name {
        public string value;
        public Location location;
        public static bool operator ==(Name name, Name nameCompare) {
            return name.value == nameCompare.value;
        }
        public static bool operator !=(Name name, Name nameCompare) {
            return name.value != nameCompare.value;
        }
    }
}
