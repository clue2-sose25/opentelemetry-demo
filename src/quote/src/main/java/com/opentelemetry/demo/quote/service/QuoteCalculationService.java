package com.opentelemetry.demo.quote.service;

import org.springframework.stereotype.Service;
import java.math.BigDecimal;
import java.math.RoundingMode;
import java.util.Random;

@Service
public class QuoteCalculationService {
    
    private final Random random = new Random();
    
    /**
     * Calculates shipping quote based on number of items.
     * Cost per item is random between $40.00 - $100.00
     * 
     * @param numberOfItems Number of items to ship
     * @return Total quote rounded to 2 decimal places
     */
    public BigDecimal calculateQuote(int numberOfItems) {
        if (numberOfItems <= 0) {
            throw new IllegalArgumentException("Number of items must be greater than 0");
        }
        
        // Generate random cost per item between 40.0 and 100.0 (equivalent to PHP rand(400, 1000)/10)
        double costPerItem = 40.0 + (random.nextDouble() * 60.0);
        
        // Calculate total and round to 2 decimal places
        double total = costPerItem * numberOfItems;
        
        return BigDecimal.valueOf(total).setScale(2, RoundingMode.HALF_UP);
    }
}
