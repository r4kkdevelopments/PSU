using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection.Statements {
    [MoonSharpUserData]
    public class NodeCompoundAssignmentStatement : NodeStatement {
        public NodeExpression variable;
        public CompoundOp operation;
        public NodeExpression value;
        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }
}
