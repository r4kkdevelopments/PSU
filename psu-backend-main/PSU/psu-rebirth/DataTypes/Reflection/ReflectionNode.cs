using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection {
    [MoonSharpUserData]
    public abstract class ReflectionNode {
        private static uint NODE_COUNT = 0;

        public ReflectionNode parent;
        public Location location;
        public uint PS_TAG;
        public ReflectionNode() {
            PS_TAG = NODE_COUNT + 1;
            NODE_COUNT = PS_TAG;
        }

        public abstract void visit(IVisitor visitor);
    }
}
