#!/usr/bin/env node

import fs from "fs";
import path from "path";

interface CoverageData {
  [key: string]: {
    path: string;
    statementMap: { [key: string]: any };
    fnMap: { [key: string]: any };
    branchMap: { [key: string]: any };
    s: { [key: string]: number };
    f: { [key: string]: number };
    b: { [key: string]: number[] };
  };
}

function checkCoverage() {
  const coveragePath = path.join(__dirname, "..", "coverage", "coverage-final.json");
  
  if (!fs.existsSync(coveragePath)) {
    console.error("Error: Coverage file not found. Run tests with coverage first.");
    process.exit(1);
  }

  const coverageData: CoverageData = JSON.parse(fs.readFileSync(coveragePath, "utf8"));
  
  const totals = Object.values(coverageData).reduce(
    (acc, file) => {
      // Statements
      const statements = Object.values(file.s);
      acc.statements.covered += statements.filter(count => count > 0).length;
      acc.statements.total += statements.length;
      
      // Branches
      const branches = Object.values(file.b).flat();
      acc.branches.covered += branches.filter(count => count > 0).length;
      acc.branches.total += branches.length;
      
      // Functions
      const functions = Object.values(file.f);
      acc.functions.covered += functions.filter(count => count > 0).length;
      acc.functions.total += functions.length;
      
      return acc;
    },
    {
      statements: { covered: 0, total: 0 },
      branches: { covered: 0, total: 0 },
      functions: { covered: 0, total: 0 }
    }
  );

  const percentages = {
    statements: (totals.statements.covered / totals.statements.total) * 100,
    branches: (totals.branches.covered / totals.branches.total) * 100,
    functions: (totals.functions.covered / totals.functions.total) * 100
  };

  console.log("\nTypeScript Coverage Summary:");
  console.log(`Statements: ${percentages.statements.toFixed(2)}% (${totals.statements.covered}/${totals.statements.total})`);
  console.log(`Branches:   ${percentages.branches.toFixed(2)}% (${totals.branches.covered}/${totals.branches.total})`);
  console.log(`Functions:  ${percentages.functions.toFixed(2)}% (${totals.functions.covered}/${totals.functions.total})`);

  const thresholds = {
    statements: 70,
    branches: 60,
    functions: 60
  };

  let failed = false;
  
  if (percentages.statements < thresholds.statements) {
    console.error(`\n✗ Statement coverage (${percentages.statements.toFixed(2)}%) is below threshold (${thresholds.statements}%)`);
    failed = true;
  }
  
  if (percentages.branches < thresholds.branches) {
    console.error(`✗ Branch coverage (${percentages.branches.toFixed(2)}%) is below threshold (${thresholds.branches}%)`);
    failed = true;
  }
  
  if (percentages.functions < thresholds.functions) {
    console.error(`✗ Function coverage (${percentages.functions.toFixed(2)}%) is below threshold (${thresholds.functions}%)`);
    failed = true;
  }

  if (failed) {
    console.error("\nCoverage check failed!");
    process.exit(1);
  } else {
    console.log("\n✓ All coverage thresholds met!");
  }
}

if (require.main === module) {
  checkCoverage();
}

export { checkCoverage };