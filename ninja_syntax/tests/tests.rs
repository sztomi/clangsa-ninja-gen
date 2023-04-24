#[cfg(test)]
mod tests {
  use ninja_syntax::{Build, Rule, Variable, Writer};
  use std::collections::HashMap;

  fn nw() -> Writer {
    Writer::new(&":memory:")
  }

  #[test]
  fn comment() {
    assert_eq!("# Hello\n", nw().comment("Hello").as_str());
  }

  #[test]
  fn newline() {
    assert_eq!("\n", nw().newline().as_str());
  }

  #[test]
  fn variable() {
    assert_eq!("foo = bar\n", nw().variable("foo", "bar", 0).as_str());
    assert_eq!("  foo = bar\n", nw().variable("foo", "bar", 1).as_str());
  }

  #[test]
  fn variable_list() {
    assert_eq!(
      "foo = bar hello world\n",
      nw()
        .variable_list("foo", &["bar", "hello", "world"], 0)
        .as_str()
    );
  }

  #[test]
  fn pool() {
    assert_eq!(
      "pool console\n  depth = 2\n",
      nw().pool("console", 2).as_str()
    );
  }

  #[test]
  fn rule() {
    let res = r#"rule cc
  command = $cc $in -o $out
"#;
    assert_eq!(res, nw().rule(&Rule::new("cc", "$cc $in -o $out")).as_str());
  }

  #[test]
  fn rule_full() {
    let res = r#"rule full
  command = full force
  description = doing full
  depfile = fullfile
  generator = 1
  pool = fullpool
  restat = 1
  rspfile = fullrsp
  rspfile_content = full_content
  deps = msvc
"#;
    assert_eq!(
      res,
      nw()
        .rule(
          &Rule::new("full", "full force")
            .description("doing full")
            .depfile("fullfile")
            .generator(true)
            .pool("fullpool")
            .restat(true)
            .rspfile("fullrsp")
            .rspfile_content("full_content")
            .deps("msvc")
        )
        .as_str()
    );
  }

  #[test]
  fn build() {
    assert_eq!(
      "build foo.o: cc foo.c\n",
      nw()
        .build(&Build::new(&["foo.o"], "cc").inputs(&["foo.c"]))
        .as_str()
    );
    assert_eq!(
      "build out$ dir/foo$:bar.o: cc in$ dir/foo.c\n",
      nw()
        .build(&Build::new(&["out dir/foo:bar.o"], "cc").inputs(&["in dir/foo.c"]))
        .as_str()
    );
  }

  #[test]
  fn build_implicit() {
    assert_eq!(
      "build foo.o: cc foo.c | foo.h\n",
      nw()
        .build(
          &Build::new(&["foo.o"], "cc")
            .inputs(&["foo.c"])
            .implicit(&["foo.h"])
        )
        .as_str()
    );
    assert_eq!(
      "build foo.o: cc foo.c || foo.h\n",
      nw()
        .build(
          &Build::new(&["foo.o"], "cc")
            .inputs(&["foo.c"])
            .order_only(&["foo.h"])
        )
        .as_str()
    );
    assert_eq!(
      "build foo.o | foo.ast: cc\n",
      nw()
        .build(&Build::new(&["foo.o"], "cc").implicit_outputs(&["foo.ast"]))
        .as_str()
    );
  }

  #[test]
  fn build_other() {
    let res = r#"build foo.o bar.o: cc foo.c bar.c
  pool = hello
  dyndep = dyndep
  cflags = -DFOO=BAR /O2
"#;
    let mut var = HashMap::new();
    var.insert("cflags", "-DFOO=BAR /O2");

    assert_eq!(
      res,
      nw()
        .build(
          &Build::new(&["foo.o", "bar.o"], "cc")
            .inputs(&["foo.c", "bar.c"])
            .dyndep("dyndep")
            .pool("hello")
            .variables(&var)
        )
        .as_str()
    );
  }

  #[test]
  fn bulk_add() {
    let builds = vec![
      Build::new(&["foo.o"], "cc").inputs(&["foo.c"]),
      Build::new(&["bar.o"], "cc").inputs(&["bar.c"]),
    ];

    assert_eq!(
      "build foo.o: cc foo.c
build bar.o: cc bar.c\n",
      nw().write_builds(&builds, false).as_str()
    );

    let rules = vec![
      Rule::new("cc", "$cc $in -o $out"),
      Rule::new("cxx", "$cxx $in -o $out"),
    ];
    assert_eq!(
      "rule cc\n  command = $cc $in -o $out\nrule cxx\n  command = $cxx $in -o $out\n",
      nw().write_rules(&rules, false).as_str()
    );

    let vars = vec![
      Variable::new("foo", "bar", 0),
      Variable::new("hello", "world", 0),
    ];
    assert_eq!(
      "foo = bar\nhello = world\n",
      nw().write_variables(&vars, false).as_str()
    );
  }
}
