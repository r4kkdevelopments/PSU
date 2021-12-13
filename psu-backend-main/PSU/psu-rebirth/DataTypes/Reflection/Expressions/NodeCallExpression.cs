using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection.Expressions {
    [MoonSharpUserData]
    public class NodeCallExpression : NodeExpression {
        public NodeExpression function;
        public List<NodeExpression> arguments = new List<NodeExpression>();
        public bool selfCall;
        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }
}
