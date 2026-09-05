# JUnit XML to CSV

Convert a JUnit/xUnit XML report to a CSV of test cases

Turn a JUnit / xUnit test report (surefire, pytest --junitxml, Jest, Go gotestsum, CTest ...) into a CSV table with one row per test case and the columns suite,classname,name,time,status. status is derived from the case's child element -- <failure> is failed, <error> is error, <skipped> is skipped, and a case with none of them passed -- so a bare self-closing <testcase/> is correctly reported as a pass. Both report shapes are handled: a single <testsuite> root and a <testsuites> wrapper holding several suites (the suite name is carried down onto its own cases). Element and attribute text is XML-unescaped and the result is RFC 4180 CSV, so test names holding commas, quotes or ampersands survive intact. Scope: attributes are read off each <testcase> element, so a report is expected to keep name/classname/time as attributes (they are, in every generator above); <properties>, <system-out>, <system-err> and failure bodies are not exported, and any line that does not resolve to a suite or case row is flagged rather than dropped. Sort by status or time to get the failing or slow tests out of a CI artifact without opening it.

## Run

```
mog -m junit-xml-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="build 4821" tests="8" failures="2" errors="1" skipped="1" time="6.412">
  <testsuite name="checkout.CartTest" tests="4" failures="1" errors="0" skipped="1" time="3.104" timestamp="2026-08-14T09:12:03">
    <properties>
      <property name="java.version" value="21.0.2"/>
    </properties>
    <testcase classname="checkout.CartTest" name="addsItemToCart" time="0.412"/>
    <testcase classname="checkout.CartTest" name="totals with tax, tip &amp; discount" time="0.907">
      <failure message="expected &lt;12.50&gt; but was &lt;12.05&gt;" type="java.lang.AssertionError">
java.lang.AssertionError: expected &lt;12.50&gt; but was &lt;12.05&gt;
        at checkout.CartTest.totals(CartTest.java:88)
      </failure>
    </testcase>
    <testcase classname="checkout.CartTest" name="rejects a &quot;negative&quot; quantity" time="0.311"/>
    <testcase classname="checkout.CartTest" name="appliesLoyaltyPoints" time="1.474">
      <skipped message="loyalty service not provisioned"/>
    </testcase>
  </testsuite>
```

_(... 13 more line(s))_

Output:

```
suite,classname,name,time,status
checkout.CartTest,checkout.CartTest,addsItemToCart,0.412,passed
checkout.CartTest,checkout.CartTest,"totals with tax, tip & discount",0.907,failed
checkout.CartTest,checkout.CartTest,"rejects a ""negative"" quantity",0.311,passed
checkout.CartTest,checkout.CartTest,appliesLoyaltyPoints,1.474,skipped
api.SearchTest,api.SearchTest,findsByKeyword,0.220,passed
api.SearchTest,api.SearchTest,paginates results,0.658,error
api.SearchTest,api.SearchTest,"sorts by price, then name",1.030,failed
api.SearchTest,api.SearchTest,escapesQueryString,1.400,passed
```

## Pipeline

- `replace_regex`: Fold the whole report onto a single line so no element is split across lines.
- `replace_regex`: Start a new line at every <testsuite> and <testcase> element.
- `keep_lines_matching`: Keep only the suite and case lines (drops the XML declaration, <testsuites> and properties).
- `replace_regex_multiline`: A case containing <failure> is failed.
- `replace_regex_multiline`: A case containing <error> is error.
- `replace_regex_multiline`: A case containing <skipped> is skipped.
- `replace_regex_multiline`: Every remaining case passed.
- `replace_regex_multiline`: Capture each case's classname attribute.
- `replace_regex_multiline`: Give a case with no classname an empty one.
- `replace_regex_multiline`: Capture each case's name attribute.
- `replace_regex_multiline`: Capture each case's time attribute.
- `replace_regex_multiline`: Give a case with no time an empty one.
- `replace_regex_multiline`: Rewrite each suite line as a suite-name row with the other columns blank.
- `replace_regex_multiline`: Rewrite each case line as a row with a blank suite column.
- `replace_regex`: Mark genuinely empty cells so the fill-down cannot invent a value for them.
- `flag_matching`: Flag any line the shapes above did not turn into a row.
- `fill_down`: Carry each suite name down onto its own cases.
- `remove_lines_matching`: Drop the suite marker rows now that their names are carried down.
- `replace`: Restore the marked empty cells.
- `html_decode`: Decode the XML entities in the suite, class and test names.
- `replace`: Double every embedded double quote.
- `replace_regex`: Wrap any field holding a comma or a quote in double quotes.
- `replace`: Swap each unit separator for a comma.
- `prepend`: Add the CSV header row.

## Tags

`junit` `xml` `csv` `convert` `testing` `ci`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->
