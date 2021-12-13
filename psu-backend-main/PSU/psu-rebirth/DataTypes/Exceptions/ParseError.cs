using psu_rebirth.DataTypes.Reflection;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace psu_rebirth.DataTypes.Exceptions {
    public class ParseError : Exception {
        public ParseError(Location location, string message) : base(constructException(location, message)) { }

        public static string constructException(Location location, string message) {
            var sameLine = location.end.line == location.start.line;
            var sameCol = sameLine && location.end.column == location.start.column;

            var errorLine = location.end.line + 1;

            var error = errorLine.ToString() + ": " + message;
            if (!sameLine)
                error += " (at line " + (location.start.line + 1) + ")";
            else if (location.start.column != 0)
                error += " (at column " + (location.start.column + 1) + ")";

            return error;
        }
    }
}
