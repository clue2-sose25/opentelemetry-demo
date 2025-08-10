package com.opentelemetry.demo.quote.dto;

import com.fasterxml.jackson.annotation.JsonProperty;

public class QuoteRequest {
    
    @JsonProperty("numberOfItems")
    private int numberOfItems;
    
    public QuoteRequest() {
    }
    
    public QuoteRequest(int numberOfItems) {
        this.numberOfItems = numberOfItems;
    }
    
    public int getNumberOfItems() {
        return numberOfItems;
    }
    
    public void setNumberOfItems(int numberOfItems) {
        this.numberOfItems = numberOfItems;
    }
}
