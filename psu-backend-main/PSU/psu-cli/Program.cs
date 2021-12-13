using psu_rebirth.Lexer;
using System;
using System.IO;
using psu_rebirth.Parser;

using Terminal.Gui;
using System.Text.Json;
using System.Text.Json.Serialization;
using Terminal.Gui.Trees;
using static Terminal.Gui.View;
using System.Diagnostics;
using psu_rebirth.DataTypes.Reflection;
using psu_rebirth.Engine;

namespace psu_cli {
    class Program {
        static TreeView astView = new TreeView() { X = 0, Y = 0, Width = Dim.Fill(), Height = Dim.Fill(1) };
        static TreeView fieldView = new TreeView() { X = 0, Y = 0, Width = Dim.Fill(), Height = Dim.Fill(1) };

        static void OpenFile(string file) {
            astView.ClearObjects();
            if (File.Exists(file)) {
                try {
                    var parser = new Parser();
                    var node = parser.parse(File.ReadAllText(file));

                    TreeVisitor visitor = new TreeVisitor(astView);
                    node.visit(visitor);
                    astView.AddObject(visitor.currentNode);

                    ForgeRunner.currentScriptBody = (NodeBody)node;
                } catch (Exception e) {
                    var info = MessageBox.Query(50, 7, "Critical Error", e.Message, "Ok");
                }
            }
        }

        private static void astView_SelectionChanged(object sender, SelectionChangedEventArgs<ITreeNode> e) {
            fieldView.ClearObjects();
            if (astView.SelectedObject != null) {
                var tag = astView.SelectedObject.Tag;
                if (tag != null) {
                    fieldView.AddObject(new TreeNode("type: " + tag.GetType().Name));
                    var fields = tag.GetType().GetFields();
                    foreach (var info in fields) {
                        TreeNode node = new TreeNode(info.Name + ": " + info.GetValue(tag));
                        fieldView.AddObject(node);
                    }
                }
            }
        }

        static void Main(string[] args) {
            astView.SelectionChanged += astView_SelectionChanged;
            Application.Init();
            var top = Application.Top;
            var window = new Window("psu-rebirth") {
                X = 0,
                Y = 1,
                Width = Dim.Fill(),
                Height = Dim.Fill()
            };
            var astWindow = new Window("psu-ast") {
                X = Pos.Percent(0f),
                Y = 1,
                Width  = Dim.Percent(70f),
                Height = Dim.Fill()
            };
            var fieldsWindow = new Window("psu-ast-fields") {
                X = Pos.Percent(70f),
                Y = 1,
                Width = Dim.Percent(30f),
                Height = Dim.Fill()
            };

            Colors.Base.Normal = Application.Driver.MakeAttribute(Color.Green, Color.Black);
            Colors.Menu.Normal = Application.Driver.MakeAttribute(Color.Green, Color.Black);
            Colors.Dialog.Normal = Application.Driver.MakeAttribute(Color.Black, Color.Green);
            top.Add(window);

            var toolMenu = new MenuBar(new MenuBarItem[] {
                new MenuBarItem("File", new MenuItem[] {
                    new MenuItem("Open", "Opens a .lua file.", () => {
                        var fileDialog = new OpenDialog("Open file", "Select a .lua file");
                        fileDialog.DirectoryPath = Environment.GetFolderPath(Environment.SpecialFolder.Desktop);
                        fileDialog.AllowsMultipleSelection = false;
                        fileDialog.AllowedFileTypes = new string[] { ".lua" };
                        Application.Run(fileDialog);
                        if (!fileDialog.Canceled) {
                            var file = fileDialog.FilePath.ToString();
                            OpenFile(file);
                        }
                    }),
                    new MenuItem("Run Forge Plugin", "Runs a .lua forge plugin.", () => {
                        var fileDialog = new OpenDialog("Open file", "Select a .lua file");
                        fileDialog.DirectoryPath = Environment.GetFolderPath(Environment.SpecialFolder.Desktop);
                        fileDialog.AllowsMultipleSelection = false;
                        fileDialog.AllowedFileTypes = new string[] { ".lua" };
                        Application.Run(fileDialog);
                        if (!fileDialog.Canceled) {
                            var file = fileDialog.FilePath.ToString();
                            var source = File.ReadAllText(file);
                            //try {
                                ForgeRunner.runForgePlugin(source);
                            //} catch (Exception e) {
                                //var info = MessageBox.Query(50, 7, "Critical Error", e.Message, "Ok");
                            //}
                        }
                    }),
                    new MenuItem("Close", "Closes current psu-rebirth session.", () => {

                    }),
                    new MenuItem("Quit", "Closes current psu-rebirth terminal session.", () => {
                        var option = MessageBox.Query(50, 7, "Quit", "Are you sure you want to quit?", "Yes", "No");
                        if (option == 0)
                            top.Running = false;
                    })
                })
            });
            top.Add(toolMenu);

            astWindow.Add(astView);
            fieldsWindow.Add(fieldView);
            window.Add(astWindow);
            window.Add(fieldsWindow);
            Application.Run();
        }
    }
}
