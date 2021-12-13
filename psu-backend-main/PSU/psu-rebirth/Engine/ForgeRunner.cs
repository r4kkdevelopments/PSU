using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;
using MoonSharp.Interpreter;
using MoonSharp.Interpreter.Interop;
using psu_rebirth.DataTypes.Reflection;

namespace psu_rebirth.Engine {
    public static class ForgeRunner {
        public static NodeBody currentScriptBody = null;
        public static DynValue runForgePlugin(string script = "") {
            var scriptObject = new Script(CoreModules.Preset_HardSandbox);
            UserData.RegisterAssembly(Assembly.GetAssembly(typeof(ForgeRunner)));
            scriptObject.Options.DebugPrint = s => { Debug.WriteLine(s); };
            scriptObject.Globals["forge"] = UserData.Create(new ForgeAnalyticalEngine(scriptObject));
            scriptObject.Globals["_AST"] = currentScriptBody;
            return scriptObject.DoString(script);
        }
    }
}
