using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection.Expressions {
    [MoonSharpUserData]
    public class NodeLocalExpression : NodeExpression {
        public LocalData local;

        public bool isUpvalue;

        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }
}
