using System.Collections.Generic;
using MoonSharp.Interpreter;

namespace psu_rebirth.DataTypes.Reflection.Statements {
    [MoonSharpUserData]
    public class NodeIfStatement : NodeStatement {
        public NodeExpression condition;
        public NodeBody conditionBody;
        public NodeStatement elseBody;
        public List<NodeStatement> elseIfBodies = new List<NodeStatement>();
        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }
}