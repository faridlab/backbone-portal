# Feature template — declarative, business-level scenarios.
# One Scenario per business rule or path. One When per scenario.
# Keep UI selectors / HTTP verbs OUT of step text — they belong in step definitions.

Feature: <capability in business terms>
  In order to <business value>
  As a <actor / role>
  I want to <goal>

  Background:
    Given <shared precondition for every scenario below>

  @happy-path @module:<module>
  Scenario: <main path, success>
    Given <precondition state>
    When <the single trigger, in business language>
    Then <observable postcondition>
    And <further observable postcondition>

  @edge @module:<module>
  Scenario: <exception path — a business rule blocks the flow>
    Given <precondition that violates a rule>
    When <the trigger>
    Then <the flow is rejected with the business reason>

  @edge @module:<module>
  Scenario Outline: <same rule across boundary values>
    Given <state parameterized by the example>
    When <the trigger>
    Then <expected business outcome for that value>

    Examples:
      | <param> | <expected> |
      | <...>   | <...>      |
