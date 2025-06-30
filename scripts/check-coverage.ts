#!/usr/bin/env node

import * as fs from "fs";
import * as path from "path";
import { execSync } from "child_process";

// ANSI color codes
const colors = {
  reset: "\x1b[0m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  gray: "\x1b[90m",
  bold: "\x1b[1m",
};

function colorize(text: string, color: keyof typeof colors): string {
  return `${colors[color]}${text}${colors.reset}`;
}

function getColorForPercentage(pct: number, threshold: number): string {
  if (pct >= threshold) return colors.green;
  if (pct >= threshold * 0.9) return colors.yellow;
  return colors.red;
}

interface CoverageData {
  [key: string]: {
    path: string;
    statementMap: { [key: string]: unknown };
    fnMap: { [key: string]: unknown };
    branchMap: { [key: string]: unknown };
    s: { [key: string]: number };
    f: { [key: string]: number };
    b: { [key: string]: number[] };
  };
}

interface FileCoverage {
  path: string;
  lines: { covered: number; total: number; pct: number };
  statements: { covered: number; total: number; pct: number };
  branches: { covered: number; total: number; pct: number };
  functions: { covered: number; total: number; pct: number };
}

interface TarpaulinResult {
  lines: { covered: number; total: number; percentage: number };
  branches: { covered: number; total: number; percentage: number };
  functions: { covered: number; total: number; percentage: number };
}

interface TarpaulinTrace {
  line: number;
  stats?: {
    Line?: number;
  };
}

interface TarpaulinFunction {
  start: number;
  end: number;
}

interface TarpaulinJson {
  traces?: {
    [filePath: string]: TarpaulinTrace[];
  };
  functions?: {
    [filePath: string]: TarpaulinFunction[];
  };
}

function formatFilePath(path: string, maxLength: number = 35): string {
  const relativePath = path.replace(/^.*\/lintp\//, "");
  if (relativePath.length <= maxLength) {
    return relativePath.padEnd(maxLength, ".");
  }
  return `...${relativePath.slice(-(maxLength - 3))}`;
}

function formatPercentage(pct: number): string {
  return pct === 100 ? "100%" : `${pct.toFixed(1)}%`;
}

async function runTypeScriptCoverage(showTable: boolean = true): Promise<boolean> {
  try {
    console.log("Running TypeScript coverage...");
    
    // Run vitest with coverage
    execSync("./node_modules/.bin/vitest run --coverage", { 
      stdio: ["ignore", "ignore", "inherit"] 
    });
    
    // Read the coverage data
    const coveragePath = path.join(__dirname, "..", "coverage", "coverage-final.json");
    if (!fs.existsSync(coveragePath)) {
      console.error("Error: Coverage file not found. Run tests with coverage first.");
      return true;
    }
    
    const coverageData: CoverageData = JSON.parse(fs.readFileSync(coveragePath, "utf8"));
    
    // Calculate per-file coverage
    const fileCoverages: FileCoverage[] = [];
    const totals = {
      statements: { covered: 0, total: 0 },
      branches: { covered: 0, total: 0 },
      functions: { covered: 0, total: 0 },
      lines: { covered: 0, total: 0 }
    };
    
    for (const [, file] of Object.entries(coverageData)) {
      // Statements
      const statements = Object.values(file.s);
      const stmtCovered = statements.filter(count => count > 0).length;
      const stmtTotal = statements.length;
      
      // Branches
      const branches = Object.values(file.b).flat();
      const branchCovered = branches.filter(count => count > 0).length;
      const branchTotal = branches.length;
      
      // Functions
      const functions = Object.values(file.f);
      const funcCovered = functions.filter(count => count > 0).length;
      const funcTotal = functions.length;
      
      // Lines are the same as statements in v8 coverage
      const lineCovered = stmtCovered;
      const lineTotal = stmtTotal;
      
      fileCoverages.push({
        path: file.path,
        lines: { 
          covered: lineCovered, 
          total: lineTotal, 
          pct: lineTotal > 0 ? (lineCovered / lineTotal) * 100 : 100 
        },
        statements: { 
          covered: stmtCovered, 
          total: stmtTotal, 
          pct: stmtTotal > 0 ? (stmtCovered / stmtTotal) * 100 : 100 
        },
        branches: { 
          covered: branchCovered, 
          total: branchTotal, 
          pct: branchTotal > 0 ? (branchCovered / branchTotal) * 100 : 100 
        },
        functions: { 
          covered: funcCovered, 
          total: funcTotal, 
          pct: funcTotal > 0 ? (funcCovered / funcTotal) * 100 : 100 
        }
      });
      
      // Update totals
      totals.statements.covered += stmtCovered;
      totals.statements.total += stmtTotal;
      totals.branches.covered += branchCovered;
      totals.branches.total += branchTotal;
      totals.functions.covered += funcCovered;
      totals.functions.total += funcTotal;
    }
    
    // Lines are the same as statements in v8 coverage
    totals.lines = totals.statements;
    
    // Calculate percentages
    const pct = {
      statements: { pct: (totals.statements.covered / totals.statements.total) * 100 },
      branches: { pct: (totals.branches.covered / totals.branches.total) * 100 },
      functions: { pct: (totals.functions.covered / totals.functions.total) * 100 },
      lines: { pct: (totals.lines.covered / totals.lines.total) * 100 }
    };
    
    if (showTable) {
      console.log(`\n${colorize("TypeScript Coverage:", "bold")}`);
      
      // Sort files by path
      fileCoverages.sort((a, b) => a.path.localeCompare(b.path));
      
      // Display per-file coverage
      for (const file of fileCoverages) {
        const lineColor = getColorForPercentage(file.lines.pct, 70);
        const branchColor = getColorForPercentage(file.branches.pct, 60);
        const funcColor = getColorForPercentage(file.functions.pct, 60);
        const stmtColor = getColorForPercentage(file.statements.pct, 70);
        
        const formattedPath = formatFilePath(file.path);
        const line = `  ${formattedPath} L: ${lineColor}${formatPercentage(file.lines.pct)}${colors.reset} | B: ${branchColor}${formatPercentage(file.branches.pct)}${colors.reset} | F: ${funcColor}${formatPercentage(file.functions.pct)}${colors.reset} | S: ${stmtColor}${formatPercentage(file.statements.pct)}${colors.reset}`;
        console.log(line);
      }
    }
    
    const thresholds = {
      statements: 70,
      branches: 60,
      functions: 60,
      lines: 70
    };
    
    let failed = false;
    
    if (pct.statements.pct < thresholds.statements) {
      console.error(colorize(`\n✗ Statement coverage (${pct.statements.pct.toFixed(2)}%) is below threshold (${thresholds.statements}%)`, "red"));
      failed = true;
    }
    
    if (pct.branches.pct < thresholds.branches) {
      console.error(colorize(`✗ Branch coverage (${pct.branches.pct.toFixed(2)}%) is below threshold (${thresholds.branches}%)`, "red"));
      failed = true;
    }
    
    if (pct.functions.pct < thresholds.functions) {
      console.error(colorize(`✗ Function coverage (${pct.functions.pct.toFixed(2)}%) is below threshold (${thresholds.functions}%)`, "red"));
      failed = true;
    }
    
    if (pct.lines.pct < thresholds.lines) {
      console.error(colorize(`✗ Line coverage (${pct.lines.pct.toFixed(2)}%) is below threshold (${thresholds.lines}%)`, "red"));
      failed = true;
    }
    
    if (!failed) {
      console.log(colorize("\n✓ TypeScript coverage thresholds met!", "green"));
    }
    
    return failed;
  } catch (error) {
    console.error("Error running TypeScript coverage:", error);
    return true;
  }
}

async function runRustCoverage(verbose: boolean = false): Promise<boolean> {
  try {
    console.log("\nRunning Rust coverage...");
    
    // Run cargo tarpaulin with JSON output to a file
    const jsonPath = path.join(__dirname, "..", "target", "tarpaulin", "lintp-coverage.json");
    const cmd = verbose 
      ? `cargo tarpaulin --out Json --output-dir target/tarpaulin --verbose` 
      : `cargo tarpaulin --out Json --output-dir target/tarpaulin`;
      
    execSync(cmd, { stdio: ["ignore", "ignore", verbose ? "inherit" : "ignore"] });
    
    // Read and parse the JSON output
    let result: TarpaulinResult;
    try {
      const jsonOutput: TarpaulinJson = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
      
      // The tarpaulin JSON format has 'traces' at the top level with file paths as keys
      const lines = { covered: 0, total: 0 };
      const branches = { covered: 0, total: 0 };
      const functions = { covered: 0, total: 0 };
      
      // Calculate per-file stats
      interface FileData {
        covered: number;
        total: number;
        percentage: number;
      }
      
      const fileStats: Map<string, FileData> = new Map();
      
      // Extract coverage data from tarpaulin JSON format
      if (jsonOutput.traces) {
        Object.entries(jsonOutput.traces).forEach(([filePath, traces]) => {
          const relativePath = filePath.replace(/^.*\/src\//, "src/");
          let fileCovered = 0;
          let fileTotal = 0;
          
          traces.forEach((trace) => {
            if (trace.stats && trace.stats.Line !== undefined) {
              fileTotal++;
              lines.total++;
              if (trace.stats.Line > 0) {
                fileCovered++;
                lines.covered++;
              }
            }
          });
          
          if (fileTotal > 0) {
            fileStats.set(relativePath, {
              covered: fileCovered,
              total: fileTotal,
              percentage: (fileCovered / fileTotal) * 100
            });
          }
        });
      }
      
      // Extract function data from the functions map
      if (jsonOutput.functions) {
        Object.entries(jsonOutput.functions).forEach(([filePath, funcs]) => {
          funcs.forEach((func) => {
            functions.total++;
            // Check if the function has been executed (has non-zero line count in any of its lines)
            const funcStart = func.start;
            const funcEnd = func.end;
            let executed = false;
            
            if (jsonOutput.traces && jsonOutput.traces[filePath]) {
              jsonOutput.traces[filePath].forEach((trace) => {
                if (trace.line >= funcStart && trace.line <= funcEnd && 
                    trace.stats && trace.stats.Line > 0) {
                  executed = true;
                }
              });
            }
            
            if (executed) {
              functions.covered++;
            }
          });
        });
      }
      
      result = {
        lines: {
          covered: lines.covered,
          total: lines.total,
          percentage: lines.total > 0 ? (lines.covered / lines.total) * 100 : 0
        },
        branches: {
          covered: branches.covered,
          total: branches.total,
          percentage: branches.total > 0 ? (branches.covered / branches.total) * 100 : 0
        },
        functions: {
          covered: functions.covered,
          total: functions.total,
          percentage: functions.total > 0 ? (functions.covered / functions.total) * 100 : 0
        }
      };
      
      // Display coverage
      console.log(`\n${colorize("Rust Coverage:", "bold")}`);
      
      // Sort files by path
      const sortedFiles = Array.from(fileStats.entries()).sort((a, b) => a[0].localeCompare(b[0]));
      
      // Display per-file coverage
      for (const [file, stats] of sortedFiles) {
        const lineColor = getColorForPercentage(stats.percentage, 60);
        const formattedPath = formatFilePath(file);
        // Rust tarpaulin only provides line coverage
        const line = `  ${formattedPath} L: ${lineColor}${formatPercentage(stats.percentage)}${colors.reset}`;
        console.log(line);
      }
      
    } catch (e) {
      console.error("Could not parse Rust coverage JSON output:", e);
      return true;
    }
    
    const threshold = 60;
    let failed = false;
    
    if (result.lines.percentage < threshold) {
      console.error(colorize(`\n✗ Rust line coverage (${result.lines.percentage.toFixed(2)}%) is below threshold (${threshold}%)`, "red"));
      failed = true;
    }
    
    if (!failed) {
      console.log(colorize(`\n✓ Rust coverage threshold met!`, "green"));
    }
    
    return failed;
  } catch (error) {
    console.error("Error running Rust coverage:", error);
    return true;
  }
}

async function main() {
  const args = process.argv.slice(2);
  const verbose = args.includes("--verbose") || args.includes("-v");
  const runTsOnly = args.includes("--ts");
  const runRustOnly = args.includes("--rust");
  
  let failed = false;
  
  if (!runRustOnly) {
    // Check TypeScript coverage
    const tsFailed = await runTypeScriptCoverage(true);
    failed = failed || tsFailed;
  }
  
  if (!runTsOnly) {
    // Check Rust coverage
    const rustFailed = await runRustCoverage(verbose);
    failed = failed || rustFailed;
  }
  
  if (failed) {
    console.error(colorize("\nCoverage check failed!", "red"));
    process.exit(1);
  } else {
    console.log(colorize("\n✓ All coverage thresholds met!", "green"));
  }
}

if (require.main === module) {
  main().catch(err => {
    console.error(err);
    process.exit(1);
  });
}

export { runTypeScriptCoverage, runRustCoverage };