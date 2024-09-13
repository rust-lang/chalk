(function() {var implementors = {
"chalk_integration":[["impl <a class=\"trait\" href=\"chalk_ir/trait.UnificationDatabase.html\" title=\"trait chalk_ir::UnificationDatabase\">UnificationDatabase</a>&lt;<a class=\"struct\" href=\"chalk_integration/interner/struct.ChalkIr.html\" title=\"struct chalk_integration::interner::ChalkIr\">ChalkIr</a>&gt; for <a class=\"struct\" href=\"chalk_integration/db/struct.ChalkDatabase.html\" title=\"struct chalk_integration::db::ChalkDatabase\">ChalkDatabase</a>"],["impl <a class=\"trait\" href=\"chalk_ir/trait.UnificationDatabase.html\" title=\"trait chalk_ir::UnificationDatabase\">UnificationDatabase</a>&lt;<a class=\"struct\" href=\"chalk_integration/interner/struct.ChalkIr.html\" title=\"struct chalk_integration::interner::ChalkIr\">ChalkIr</a>&gt; for <a class=\"struct\" href=\"chalk_integration/program/struct.Program.html\" title=\"struct chalk_integration::program::Program\">Program</a>"]],
"chalk_solve":[["impl&lt;I, DB, P&gt; <a class=\"trait\" href=\"chalk_ir/trait.UnificationDatabase.html\" title=\"trait chalk_ir::UnificationDatabase\">UnificationDatabase</a>&lt;I&gt; for <a class=\"struct\" href=\"chalk_solve/logging_db/struct.LoggingRustIrDatabase.html\" title=\"struct chalk_solve::logging_db::LoggingRustIrDatabase\">LoggingRustIrDatabase</a>&lt;I, DB, P&gt;<div class=\"where\">where\n    DB: <a class=\"trait\" href=\"chalk_solve/trait.RustIrDatabase.html\" title=\"trait chalk_solve::RustIrDatabase\">RustIrDatabase</a>&lt;I&gt;,\n    P: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;DB&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,\n    I: <a class=\"trait\" href=\"chalk_ir/interner/trait.Interner.html\" title=\"trait chalk_ir::interner::Interner\">Interner</a>,</div>"],["impl&lt;I, W, DB, P&gt; <a class=\"trait\" href=\"chalk_ir/trait.UnificationDatabase.html\" title=\"trait chalk_ir::UnificationDatabase\">UnificationDatabase</a>&lt;I&gt; for <a class=\"struct\" href=\"chalk_solve/logging_db/struct.WriteOnDropRustIrDatabase.html\" title=\"struct chalk_solve::logging_db::WriteOnDropRustIrDatabase\">WriteOnDropRustIrDatabase</a>&lt;I, W, DB, P&gt;<div class=\"where\">where\n    I: <a class=\"trait\" href=\"chalk_ir/interner/trait.Interner.html\" title=\"trait chalk_ir::interner::Interner\">Interner</a>,\n    W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>,\n    DB: <a class=\"trait\" href=\"chalk_solve/trait.RustIrDatabase.html\" title=\"trait chalk_solve::RustIrDatabase\">RustIrDatabase</a>&lt;I&gt;,\n    P: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/borrow/trait.Borrow.html\" title=\"trait core::borrow::Borrow\">Borrow</a>&lt;DB&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.81.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div>"],["impl&lt;I: <a class=\"trait\" href=\"chalk_ir/interner/trait.Interner.html\" title=\"trait chalk_ir::interner::Interner\">Interner</a>, DB: <a class=\"trait\" href=\"chalk_solve/trait.RustIrDatabase.html\" title=\"trait chalk_solve::RustIrDatabase\">RustIrDatabase</a>&lt;I&gt;&gt; <a class=\"trait\" href=\"chalk_ir/trait.UnificationDatabase.html\" title=\"trait chalk_ir::UnificationDatabase\">UnificationDatabase</a>&lt;I&gt; for <a class=\"struct\" href=\"chalk_solve/display/stub/struct.StubWrapper.html\" title=\"struct chalk_solve::display::stub::StubWrapper\">StubWrapper</a>&lt;'_, DB&gt;"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()