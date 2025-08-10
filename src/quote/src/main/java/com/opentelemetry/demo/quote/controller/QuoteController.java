package com.opentelemetry.demo.quote.controller;

import com.opentelemetry.demo.quote.dto.QuoteRequest;
import com.opentelemetry.demo.quote.service.QuoteCalculationService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.math.BigDecimal;

@RestController
public class QuoteController {
    
    private final QuoteCalculationService quoteCalculationService;
    
    @Autowired
    public QuoteController(QuoteCalculationService quoteCalculationService) {
        this.quoteCalculationService = quoteCalculationService;
    }
    
    /**
     * Calculate shipping quote endpoint
     * 
     * @param request Quote request containing numberOfItems
     * @return Quote amount as decimal number
     */
    @PostMapping("/getquote")
    public ResponseEntity<BigDecimal> getQuote(@RequestBody QuoteRequest request) {
        try {
            BigDecimal quote = quoteCalculationService.calculateQuote(request.getNumberOfItems());
            return ResponseEntity.ok(quote);
        } catch (IllegalArgumentException e) {
            return ResponseEntity.badRequest().build();
        }
    }
    
    /**
     * Health check endpoint for Knative
     */
    @GetMapping("/health")
    public ResponseEntity<String> health() {
        return ResponseEntity.ok("OK");
    }
}
