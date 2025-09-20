# Session 02 - RSB LESSONS INTEGRATION (URGENT)

## 🚨 CRITICAL DISCOVERY: RSB Lessons Document
Just discovered `RSB_LESSONS.md` from Cage project showing **90% code reduction** patterns!

## 🎯 IMMEDIATE HIGH-IMPACT OPPORTUNITIES

### 1. **Main.rs Refactor with dispatch!()**
Current 121 lines → **~15 lines possible**

```rust
// CURRENT: Manual argument handling
if args.generate_config {
    generate_example_config(&args.config)?;
    return Ok(());
}

// RSB PATTERN: Use dispatch!()
dispatch!(&args, {
    "start" => cmd_start,
    "generate-config" => cmd_generate_config,
    "help" => cmd_help,
    "version" => cmd_version
});
```

### 2. **Global Context Integration**
Replace manual config overrides:

```rust
// CURRENT: Manual override
if let Some(port) = args.listen_port {
    config.listen_port = port;
}

// RSB PATTERN: Global context
options!(&args);  // Auto-populates opt_*
if has_var("opt_listen_port") {
    config.listen_port = get_var("opt_listen_port").parse().unwrap();
}
```

### 3. **Output Enhancement**
```rust
// CURRENT: Basic println!
println!("🚀 Porty v{} starting up", env!("CARGO_PKG_VERSION"));

// RSB PATTERN: Use echo!()
echo!("🚀 Porty v{} starting up", env!("CARGO_PKG_VERSION"));
```

### 4. **Free Professional Features**
Built-in commands work automatically:
- `porty help` - Colored, formatted help
- `porty inspect` - Function registry
- `porty stack` - Call stack debugging

## 📋 PRIORITY TASKS (Updated)

1. **🔥 HIGH**: Apply RSB dispatch!() pattern - **90% code reduction**
2. **🔥 HIGH**: Leverage global context with opt_* variables
3. **🔥 HIGH**: Replace println! with echo!()/stderr!() macros
4. **🔥 HIGH**: Wire up HTTP dynamic routing (original goal)
5. **📊 MED**: Add comprehensive RSB tests (follow Cage patterns)
6. **📊 MED**: Add built-in commands support

## 🎯 EXPECTED OUTCOMES

- **Main.rs**: 121 lines → ~15 lines (**88% reduction**)
- **Features**: Built-in help, inspect, stack commands
- **UX**: Professional CLI experience with zero extra code
- **Maintainability**: Declarative patterns, global context

## 📁 KEY FILES TO MODIFY

1. `src/main.rs` - Apply dispatch!() pattern
2. `src/args.rs` - Simplify to just RSB Args wrapper
3. Tests - Add comprehensive RSB integration tests
4. `src/net.rs:run_route()` - Still needs HTTP integration

## 🚀 RSB POWER UNLOCKED

The Cage project proved RSB can achieve:
- **90% code reduction** (500 → 50 lines)
- **Enhanced functionality** (built-in commands)
- **Professional UX** (colored help, debugging tools)
- **Faster compilation** (fewer dependencies)

**Porty is positioned for the same transformation!**

## Next Session Priority:
**APPLY RSB LESSONS FIRST** - Then complete HTTP integration
The architectural improvements will make HTTP integration cleaner.