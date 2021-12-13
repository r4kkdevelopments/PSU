using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace psu_rebirth.DataTypes.Reflection {
    public class NodeBody : NodeStatement {
        public List<NodeStatement> body = new List<NodeStatement>();
        public override void visit(IVisitor visitor) {
            visitor.visit(this);
        }
    }
}
